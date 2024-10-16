pub mod kpi;
pub mod mission;

use crate::db::models::*;
use crate::db::schema::*;
use crate::{APIResponse, AppState, DbPool};
use actix_web::{
    get,
    web::{self, Data, Json},
};
use diesel::prelude::*;
use kpi::CachedGlobalKPIState;
use log::error;
use mission::{MissionCachedInfo, MissionKPICachedInfo};
use redis::Commands;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Serialize, Deserialize)]
pub struct APICache {
    pub time: String,
}

#[get("/update_mission_raw")]
async fn update_mission_raw_cache(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<APICache>> {
    let mapping = app_state.mapping.lock().unwrap();

    let entity_blacklist_set = mapping.entity_blacklist_set.clone();
    let entity_combine = mapping.entity_combine.clone();
    let weapon_combine = mapping.weapon_combine.clone();

    let result = web::block(move || {
        let begin = Instant::now();
        let mut db_conn = match db_pool.get() {
            Ok(conn) => conn,
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
        let result = match MissionCachedInfo::from_db_all(
            &mut db_conn,
            &entity_blacklist_set,
            &entity_combine,
            &weapon_combine,
        ) {
            Ok(x) => x,
            Err(()) => {
                error!("cannot update mission raw cache");
                return Err(());
            }
        };

        for cached_info in result {
            let seralized = rmp_serde::to_vec(&cached_info).unwrap();
            if let Err(e) = redis_conn.set::<String, Vec<u8>, ()>(
                format!("mission_raw:{}", cached_info.mission_info.id),
                seralized,
            ) {
                error!("cannot write data to redis: {}", e);
                return Err(());
            }
        }

        let _ = redis::cmd("SAVE").exec(&mut redis_conn);

        Ok(begin.elapsed())
    })
    .await
    .unwrap();

    match result {
        Ok(d) => Json(APIResponse::ok(APICache {
            time: format!("{:?}", d),
        })),

        Err(()) => Json(APIResponse::internal_error()),
    }
}

#[get("/update_mission_kpi_raw")]
async fn update_mission_kpi_cache(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<APICache>> {
    let mapping = app_state.mapping.lock().unwrap();

    let entity_blacklist_set = mapping.entity_blacklist_set.clone();
    let entity_combine = mapping.entity_combine.clone();
    let weapon_combine = mapping.weapon_combine.clone();

    let scout_special_player_set = mapping.scout_special_player_set.clone();

    drop(mapping);

    let kpi_config = match app_state.kpi_config.lock().unwrap().clone() {
        Some(x) => x,
        None => {
            return Json(APIResponse::config_required("kpi_config"));
        }
    };

    let result = web::block(move || {
        let begin = Instant::now();
        let mut db_conn = match db_pool.get() {
            Ok(conn) => conn,
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

        let character_list = match character::table
            .select(Character::as_select())
            .load(&mut db_conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get character list from db: {}", e);
                return Err(());
            }
        };

        let character_id_to_game_id = character_list
            .into_iter()
            .map(|character| (character.id, character.character_game_id))
            .collect::<HashMap<_, _>>();

        let player_list = match player::table.select(Player::as_select()).load(&mut db_conn) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get player list from db: {}", e);
                return Err(());
            }
        };

        let player_id_to_game_id = player_list
            .into_iter()
            .map(|player| (player.id, player.player_name))
            .collect::<HashMap<_, _>>();

        let result = match MissionKPICachedInfo::from_redis_all(
            &mut db_conn,
            &mut redis_conn,
            &entity_blacklist_set,
            &entity_combine,
            &weapon_combine,
            &character_id_to_game_id,
            &player_id_to_game_id,
            &scout_special_player_set,
            &kpi_config,
        ) {
            Ok(x) => x,
            Err(()) => {
                error!("cannot update mission kpi cache");
                return Err(());
            }
        };

        for cached_info in result {
            let seralized = rmp_serde::to_vec(&cached_info).unwrap();
            if let Err(e) = redis_conn.set::<String, Vec<u8>, ()>(
                format!("mission_kpi_raw:{}", cached_info.mission_id),
                seralized,
            ) {
                error!("cannot write data to redis: {}", e);
                return Err(());
            }
        }

        let _ = redis::cmd("SAVE").exec(&mut redis_conn);

        Ok(begin.elapsed())
    })
    .await
    .unwrap();

    match result {
        Ok(d) => Json(APIResponse::ok(APICache {
            time: format!("{:?}", d),
        })),

        Err(()) => Json(APIResponse::internal_error()),
    }
}

#[get("/update_global_kpi_state")]
async fn update_global_kpi_state(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<APICache>> {
    let mapping = app_state.mapping.lock().unwrap();

    let entity_blacklist_set = mapping.entity_blacklist_set.clone();
    let entity_combine = mapping.entity_combine.clone();
    let weapon_combine = mapping.weapon_combine.clone();

    let scout_special_player_set = mapping.scout_special_player_set.clone();

    drop(mapping);

    let kpi_config = match app_state.kpi_config.lock().unwrap().clone() {
        Some(x) => x,
        None => {
            return Json(APIResponse::config_required("kpi_config"));
        }
    };

    let result = web::block(move || {
        let begin = Instant::now();
        let mut db_conn = match db_pool.get() {
            Ok(conn) => conn,
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

        let character_list = match character::table
            .select(Character::as_select())
            .load(&mut db_conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get character list from db: {}", e);
                return Err(());
            }
        };

        let character_id_to_game_id = character_list
            .into_iter()
            .map(|character| (character.id, character.character_game_id))
            .collect::<HashMap<_, _>>();

        let player_list = match player::table.select(Player::as_select()).load(&mut db_conn) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get player list from db: {}", e);
                return Err(());
            }
        };

        let player_id_to_name = player_list
            .into_iter()
            .map(|player| (player.id, player.player_name))
            .collect::<HashMap<_, _>>();

        let invalid_mission_list = match mission_invalid::table
            .select(MissionInvalid::as_select())
            .load(&mut db_conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get invalid mission list from db: {}", e);
                return Err(());
            }
        };

        let invalid_mission_id_list = invalid_mission_list
            .into_iter()
            .map(|x| x.mission_id)
            .collect::<Vec<_>>();

        let result = match CachedGlobalKPIState::from_redis_all(
            &mut db_conn,
            &mut redis_conn,
            &entity_blacklist_set,
            &entity_combine,
            &weapon_combine,
            &invalid_mission_id_list,
            kpi_config,
            &player_id_to_name,
            &character_id_to_game_id,
            &scout_special_player_set,
        ) {
            Ok(x) => x,
            Err(()) => {
                error!("cannot update global kpi state");
                return Err(());
            }
        };

        let seralized = rmp_serde::to_vec(&result).unwrap();
        if let Err(e) = redis_conn.set::<&str, Vec<u8>, ()>("global_kpi_state", seralized) {
            error!("cannot write data to redis: {}", e);
            return Err(());
        }

        let _ = redis::cmd("SAVE").exec(&mut redis_conn);

        Ok(begin.elapsed())
    })
    .await
    .unwrap();

    match result {
        Ok(d) => Json(APIResponse::ok(APICache {
            time: format!("{:?}", d),
        })),

        Err(()) => Json(APIResponse::internal_error()),
    }
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(update_mission_raw_cache);
    cfg.service(update_mission_kpi_cache);
    cfg.service(update_global_kpi_state);
}
