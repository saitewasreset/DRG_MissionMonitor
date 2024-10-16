use super::{MissionTypeData, MissionTypeInfo};
use crate::cache::mission::MissionCachedInfo;
use crate::db::models::MissionType;
use crate::db::schema::*;
use crate::hazard_id_to_real;
use crate::{APIResponse, AppState, DbPool};
use actix_web::{
    get,
    web::{self, Data, Json},
};
use diesel::prelude::*;
use log::{debug, error};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[get("/mission_type")]
async fn get_mission_type(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<MissionTypeInfo>> {
    let mapping = app_state.mapping.lock().unwrap();

    let entity_blacklist_set = mapping.entity_blacklist_set.clone();
    let entity_combine = mapping.entity_combine.clone();
    let weapon_combine = mapping.weapon_combine.clone();
    let mission_type_game_id_to_name = mapping.mission_type_mapping.clone();

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

        let mission_type_list = match mission_type::table
            .select(MissionType::as_select())
            .load(&mut db_conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get mission type list from db: {}", e);
                return Err(());
            }
        };

        let mission_type_id_to_game_id = mission_type_list
            .into_iter()
            .map(|item| (item.id, item.mission_type_game_id))
            .collect::<HashMap<_, _>>();

        debug!("data prepared in {:?}", begin.elapsed());
        let begin = Instant::now();

        let result = generate(
            &cached_mission_list,
            &invalid_mission_id_list,
            &mission_type_id_to_game_id,
            mission_type_game_id_to_name,
        );

        debug!("mission type info generated in {:?}", begin.elapsed());

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
    mission_type_id_to_game_id: &HashMap<i16, String>,
    mission_type_game_id_to_name: HashMap<String, String>,
) -> MissionTypeInfo {
    let invalid_mission_id_set = invalid_mission_id_list
        .iter()
        .copied()
        .collect::<HashSet<_>>();

    let cached_mission_list = cached_mission_list
        .into_iter()
        .filter(|info| !invalid_mission_id_set.contains(&info.mission_info.id))
        .collect::<Vec<_>>();

    let mut mission_list_by_type: HashMap<i16, Vec<&MissionCachedInfo>> = HashMap::new();

    let mut result = HashMap::with_capacity(mission_list_by_type.len());

    for mission in cached_mission_list {
        let mission_type = mission.mission_info.mission_type_id;
        mission_list_by_type
            .entry(mission_type)
            .or_insert_with(Vec::new)
            .push(mission);
    }

    for (mission_type_id, mission_list) in mission_list_by_type {
        let total_difficulty = mission_list
            .iter()
            .map(|item| hazard_id_to_real(item.mission_info.hazard_id))
            .sum::<f64>();

        let total_mission_time = mission_list
            .iter()
            .map(|item| item.mission_info.mission_time as i32)
            .sum::<i32>();

        let total_reward_credit = mission_list
            .iter()
            .map(|item| item.mission_info.reward_credit)
            .sum::<f64>();

        let pass_count = mission_list
            .iter()
            .filter(|item| item.mission_info.result == 0)
            .count();
        let mission_count = mission_list.len();

        let mission_type_game_id = mission_type_id_to_game_id
            .get(&mission_type_id)
            .unwrap()
            .clone();
        result.insert(
            mission_type_game_id,
            MissionTypeData {
                average_difficulty: total_difficulty / mission_count as f64,
                average_mission_time: total_mission_time as f64 / mission_count as f64,
                average_reward_credit: total_reward_credit / mission_count as f64,
                credit_per_minute: total_reward_credit / (total_mission_time as f64 / 60.0),
                mission_count: mission_count as i32,
                pass_rate: pass_count as f64 / mission_count as f64,
            },
        );
    }

    MissionTypeInfo {
        mission_type_data: result,
        mission_type_map: mission_type_game_id_to_name,
    }
}
