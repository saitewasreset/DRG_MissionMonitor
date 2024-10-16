use super::EntityDamageInfo;
use crate::cache::mission::MissionCachedInfo;
use crate::db::schema::*;
use crate::{APIResponse, AppState, DbPool};
use actix_web::{
    get,
    web::{self, Data, Json},
};
use diesel::prelude::*;
use log::{debug, error};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[get("/entity")]
async fn get_damage_entity(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<EntityDamageInfo>> {
    let mapping = app_state.mapping.lock().unwrap();

    let entity_blacklist_set = mapping.entity_blacklist_set.clone();
    let entity_combine = mapping.entity_combine.clone();
    let weapon_combine = mapping.weapon_combine.clone();
    let entity_mapping = mapping.entity_mapping.clone();

    drop(mapping);

    let result = web::block(move || {
        let begin = Instant::now();

        let mut db_conn = match db_pool.get() {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get db connection from pool: {}", e);
                return Err(());
            }
        };

        let mut redis_conn = match redis_client.get_connection() {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get redis connection: {}", e);
                return Err(());
            }
        };

        let cached_mission_list = match MissionCachedInfo::get_cached_all(
            &mut db_conn,
            &mut redis_conn,
            &entity_blacklist_set,
            &entity_combine,
            &weapon_combine,
        ) {
            Ok(x) => x,
            Err(()) => {
                error!("cannot get cached mission list");
                return Err(());
            }
        };

        let invalid_mission_id_list: Vec<i32> = match mission_invalid::table
            .select(mission_invalid::mission_id)
            .load(&mut db_conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get invalid mission list from db: {}", e);
                return Err(());
            }
        };

        debug!("data prepared in {:?}", begin.elapsed());
        let begin = Instant::now();

        let result = generate(
            &cached_mission_list,
            &invalid_mission_id_list,
            entity_mapping,
        );

        debug!("entity damage info generated in {:?}", begin.elapsed());

        Ok(result)
    })
    .await
    .unwrap();

    match result {
        Ok(x) => Json(APIResponse::ok(x)),
        Err(()) => Json(APIResponse::internal_error()),
    }
}

fn generate(
    cached_mission_list: &[MissionCachedInfo],
    invalid_mission_id_list: &[i32],
    entity_game_id_to_name: HashMap<String, String>,
) -> EntityDamageInfo {
    let invalid_mission_id_set = invalid_mission_id_list
        .iter()
        .copied()
        .collect::<HashSet<_>>();

    let cached_mission_list = cached_mission_list
        .into_iter()
        .filter(|item| !invalid_mission_id_set.contains(&item.mission_info.id))
        .collect::<Vec<_>>();

    let mut damage_map: HashMap<&String, f64> = HashMap::new();
    let mut kill_map: HashMap<&String, i32> = HashMap::new();

    for mission in cached_mission_list {
        for data in mission.damage_info.values() {
            for (entity_game_id, pack) in data {
                if pack.taker_type != 1 {
                    let entry = damage_map.entry(entity_game_id).or_default();
                    *entry += pack.total_amount;
                }
            }
        }

        for data in mission.kill_info.values() {
            for (entity_game_id, pack) in data {
                let entry = kill_map.entry(entity_game_id).or_default();
                *entry += pack.total_amount;
            }
        }
    }

    EntityDamageInfo {
        damage: damage_map
            .into_iter()
            .map(|(k, v)| (k.clone(), v))
            .collect(),
        kill: kill_map.into_iter().map(|(k, v)| (k.clone(), v)).collect(),
        entity_mapping: entity_game_id_to_name,
    }
}
