use super::{APIMission, MissionInfo, MissionList};
use crate::cache::mission::MissionCachedInfo;
use crate::{
    db::models::{Mission, MissionInvalid, MissionType},
    db::schema::*,
    APIResponse, AppState, DbPool,
};
use actix_web::{
    get,
    web::{self, Data, Json},
};
use diesel::prelude::*;
use diesel::{RunQueryDsl, SelectableHelper};
use log::{debug, error};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[get("/api_mission_list")]
async fn get_api_mission_list(db_pool: Data<DbPool>) -> Json<APIResponse<Vec<APIMission>>> {
    let inner_pool = (*db_pool).clone();

    let mission_type_map = match web::block(|| load_mission_type_map(inner_pool))
        .await
        .unwrap()
    {
        Ok(x) => x,
        Err(()) => {
            return Json(APIResponse::internal_error());
        }
    };

    let inner_pool = (*db_pool).clone();
    let mission_list = match web::block(|| load_mission_list(inner_pool)).await.unwrap() {
        Ok(x) => x,
        Err(()) => {
            return Json(APIResponse::internal_error());
        }
    };

    let result: Vec<APIMission> = mission_list
        .into_iter()
        .map(|item| APIMission::from_mission(&mission_type_map, item))
        .collect();

    Json(APIResponse::ok(result))
}

fn load_mission_list(db_pool: Arc<DbPool>) -> Result<Vec<Mission>, ()> {
    use crate::db::schema::*;
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("cannot get db connection from pool: {}", e);
            return Err(());
        }
    };

    match mission::table.load(&mut conn) {
        Ok(data) => Ok(data),
        Err(e) => {
            error!("cannot load mission from db: {}", e);
            return Err(());
        }
    }
}

fn load_mission_type_map(db_pool: Arc<DbPool>) -> Result<HashMap<i16, String>, ()> {
    use crate::db::schema::*;
    let mut conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("cannot get db connection from pool: {}", e);
            return Err(());
        }
    };

    let mission_type_list: Vec<MissionType> = match mission_type::table.load(&mut conn) {
        Ok(x) => x,
        Err(e) => {
            error!("cannot load mission type from db: {}", e);
            return Err(());
        }
    };

    let mut table = HashMap::with_capacity(mission_type_list.len());

    for mission_type in mission_type_list {
        table.insert(mission_type.id, mission_type.mission_type_game_id);
    }

    Ok(table)
}

#[get("/mission_list")]
async fn get_mission_list(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<MissionList>> {
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

        let invalid_mission_id_list = match mission_invalid::table
            .select(MissionInvalid::as_select())
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

        debug!("mission list generated in {:?}", begin.elapsed());

        Ok(result)
    })
    .await
    .unwrap();

    match result {
        Ok(x) => Json(APIResponse::ok(x)),
        Err(()) => Json(APIResponse::internal_error()),
    }
}

pub fn generate(
    cached_mission_list: &[MissionCachedInfo],
    invalid_mission_list: &[MissionInvalid],
    mission_type_id_to_game_id: &HashMap<i16, String>,
    mission_type_game_id_to_name: HashMap<String, String>,
) -> MissionList {
    let mut mission_list = Vec::with_capacity(cached_mission_list.len());

    let invalid_mission_id_map = invalid_mission_list
        .into_iter()
        .map(|item| (item.mission_id, item))
        .collect::<HashMap<_, _>>();

    for mission in cached_mission_list {
        let current_mission_info = &mission.mission_info;

        let mission_invalid = invalid_mission_id_map.contains_key(&current_mission_info.id);
        let mission_invalid_reason = match mission_invalid {
            true => invalid_mission_id_map
                .get(&current_mission_info.id)
                .map(|item| item.reason.clone())
                .unwrap_or_else(|| "".to_string()),
            false => "".to_string(),
        };

        let mission_type_id = mission_type_id_to_game_id
            .get(&current_mission_info.mission_type_id)
            .unwrap();

        mission_list.push(MissionInfo {
            mission_id: current_mission_info.id,
            begin_timestamp: current_mission_info.begin_timestamp,
            mission_time: current_mission_info.mission_time,
            mission_type_id: mission_type_id.clone(),
            hazard_id: current_mission_info.hazard_id,
            mission_result: current_mission_info.result,
            reward_credit: current_mission_info.reward_credit,
            mission_invalid,
            mission_invalid_reason,
        });
    }

    MissionList {
        mission_info: mission_list,
        mission_type_mapping: mission_type_game_id_to_name,
    }
}
