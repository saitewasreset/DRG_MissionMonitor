use super::WeaponDamageInfo;
use crate::cache::mission::MissionCachedInfo;
use crate::db::schema::*;
use crate::{APIResponse, AppState, DbPool};
use actix_web::web;
use actix_web::{
    get,
    web::{Data, Json},
};
use diesel::prelude::*;
use log::{debug, error};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[get("/weapon")]
async fn get_damage_weapon(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<HashMap<String, WeaponDamageInfo>>> {
    let mapping = app_state.mapping.lock().unwrap();

    let weapon_game_id_to_character_game_id = mapping.weapon_character.clone();
    let weapon_mapping = mapping.weapon_mapping.clone();
    let entity_blacklist_set = mapping.entity_blacklist_set.clone();
    let entity_combine = mapping.entity_combine.clone();
    let weapon_combine = mapping.weapon_combine.clone();

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
            &weapon_game_id_to_character_game_id,
            &weapon_mapping,
        );

        debug!("weapon damage info generated in {:?}", begin.elapsed());

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
    weapon_game_id_to_character_game_id: &HashMap<String, String>,
    weapon_mapping: &HashMap<String, String>,
) -> HashMap<String, WeaponDamageInfo> {
    let invalid_mission_id_set = invalid_mission_id_list
        .iter()
        .copied()
        .collect::<HashSet<_>>();

    let cached_mission_list = cached_mission_list
        .into_iter()
        .filter(|item| !invalid_mission_id_set.contains(&item.mission_info.id))
        .collect::<Vec<_>>();

    let mut result = HashMap::new();

    for mission in cached_mission_list {
        for (weapon_game_id, pack) in &mission.weapon_damage_info {
            let damage = pack
                .detail
                .values()
                .filter(|&val| val.taker_type != 1)
                .map(|val| val.total_amount)
                .sum::<f64>();

            let friendly_fire = pack
                .detail
                .values()
                .filter(|&val| val.taker_type == 1)
                .map(|val| val.total_amount)
                .sum::<f64>();

            let hero_game_id = weapon_game_id_to_character_game_id
                .get(weapon_game_id)
                .map(|inner| inner.clone())
                .unwrap_or(String::from("Unknown"));

            let mapped_name = weapon_mapping
                .get(weapon_game_id)
                .map(|inner| inner.clone())
                .unwrap_or(weapon_game_id.clone());

            let entry = result.entry(weapon_game_id).or_insert(WeaponDamageInfo {
                damage,
                friendly_fire,
                hero_game_id,
                mapped_name,
                valid_game_count: 0,
            });

            entry.damage += damage;
            entry.friendly_fire += friendly_fire;
            entry.valid_game_count += 1;
        }
    }

    result.into_iter().map(|(k, v)| (k.clone(), v)).collect()
}
