use super::player::generate_player_kpi;
use crate::cache::kpi::CachedGlobalKPIState;
use crate::cache::mission::{MissionCachedInfo, MissionKPICachedInfo};
use crate::db::models::*;
use crate::db::schema::*;
use crate::{APIResponse, AppState, DbPool};
use crate::{KPIConfig, FLOAT_EPSILON};
use actix_web::{
    get,
    web::{self, Data, Json},
};
use diesel::prelude::*;
use log::{debug, error};
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Serialize)]
pub struct PlayerBotKPIInfo {
    #[serde(rename = "deltaPercent")]
    pub delta_percent: f64,
    pub overall: f64,
    pub recent: f64,
}

fn generate_bot_kpi_info(
    cached_mission_list: &[MissionCachedInfo],
    mission_kpi_cached_info_list: &[MissionKPICachedInfo],
    invalid_mission_id_list: &[i32],
    watchlist_player_id_list: &[i16],
    player_id_to_name: &HashMap<i16, String>,
    global_kpi_state: &CachedGlobalKPIState,
    kpi_config: &KPIConfig,
) -> HashMap<String, PlayerBotKPIInfo> {
    let player_kpi_info = generate_player_kpi(
        cached_mission_list,
        mission_kpi_cached_info_list,
        invalid_mission_id_list,
        watchlist_player_id_list,
        player_id_to_name,
        global_kpi_state,
        kpi_config,
    );

    let mut result = HashMap::with_capacity(player_kpi_info.len());

    for (player_game_id, player_info) in player_kpi_info {
        let mut player_mission_info_list = player_info
            .by_character
            .values()
            .map(|player_character_info| player_character_info.mission_list.clone())
            .flatten()
            .collect::<Vec<_>>();

        player_mission_info_list.sort_unstable_by(|a, b| a.begin_timestamp.cmp(&b.begin_timestamp));

        let prev_mission_count = match player_mission_info_list.len() * 8 / 10 {
            0..10 => 10,
            x => x,
        };

        let prev_mission_count = if prev_mission_count >= player_mission_info_list.len() {
            player_mission_info_list.len()
        } else {
            prev_mission_count
        };

        let prev_list = &player_mission_info_list[0..prev_mission_count];
        let recent_list = &player_mission_info_list[prev_mission_count..];

        let prev_player_index = prev_list.iter().map(|item| item.player_index).sum::<f64>();
        let prev_weighted_sum = prev_list
            .iter()
            .map(|item| item.mission_kpi * item.player_index)
            .sum::<f64>();

        let prev_player_kpi = prev_weighted_sum / prev_player_index;

        let overall_player_index = player_mission_info_list
            .iter()
            .map(|item| item.player_index)
            .sum::<f64>();
        let overall_weighted_sum = player_mission_info_list
            .iter()
            .map(|item| item.mission_kpi * item.player_index)
            .sum::<f64>();

        let overall_player_kpi = overall_weighted_sum / overall_player_index;

        let recent_player_index = recent_list
            .iter()
            .map(|item| item.player_index)
            .sum::<f64>();
        let recent_weighted_sum = recent_list
            .iter()
            .map(|item| item.mission_kpi * item.player_index)
            .sum::<f64>();
        let recent_player_kpi = match recent_player_index.abs() {
            0.0..FLOAT_EPSILON => overall_player_kpi,
            _ => recent_weighted_sum / recent_player_index,
        };

        let delta_percent = (recent_player_kpi - prev_player_kpi) / prev_player_kpi;

        result.insert(
            player_game_id,
            PlayerBotKPIInfo {
                delta_percent,
                overall: overall_player_kpi,
                recent: recent_player_kpi,
            },
        );
    }

    result
}

#[get("/bot_kpi_info")]
async fn get_bot_kpi_info(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<HashMap<String, PlayerBotKPIInfo>>> {
    let mapping = app_state.mapping.lock().unwrap();

    let entity_blacklist_set = mapping.entity_blacklist_set.clone();
    let entity_combine = mapping.entity_combine.clone();
    let weapon_combine = mapping.weapon_combine.clone();

    drop(mapping);

    let kpi_config = match app_state.kpi_config.lock().unwrap().clone() {
        Some(x) => x,
        None => {
            return Json(APIResponse::config_required("kpi_config"));
        }
    };

    let scout_special_player_set = app_state
        .mapping
        .lock()
        .unwrap()
        .scout_special_player_set
        .clone();

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

        let player_list = match player::table.select(Player::as_select()).load(&mut db_conn) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get player list: {}", e);
                return Err(());
            }
        };

        let watchlist_player_id_list = player_list
            .iter()
            .filter(|item| item.friend)
            .map(|item| item.id)
            .collect::<Vec<_>>();

        let player_id_to_name = player_list
            .into_iter()
            .map(|player| (player.id, player.player_name))
            .collect::<HashMap<_, _>>();

        let character_list = match character::table
            .select(Character::as_select())
            .load(&mut db_conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get character list: {}", e);
                return Err(());
            }
        };

        let character_id_to_game_id = character_list
            .into_iter()
            .map(|character| (character.id, character.character_game_id))
            .collect::<HashMap<_, _>>();

        let invalid_mission_id_list = match mission_invalid::table
            .select(mission_invalid::mission_id)
            .load::<i32>(&mut db_conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get invalid mission id list: {}", e);
                return Err(());
            }
        };

        let cached_mission_list = MissionCachedInfo::get_cached_all(
            &mut db_conn,
            &mut redis_conn,
            &entity_blacklist_set,
            &entity_combine,
            &weapon_combine,
        )?;

        let mission_kpi_cached_info_list = MissionKPICachedInfo::get_cached_all(
            &mut db_conn,
            &mut redis_conn,
            &entity_blacklist_set,
            &entity_combine,
            &weapon_combine,
            &character_id_to_game_id,
            &player_id_to_name,
            &scout_special_player_set,
            &kpi_config,
        )?;

        let global_kpi_state = CachedGlobalKPIState::get_cached(
            &mut db_conn,
            &mut redis_conn,
            &entity_blacklist_set,
            &entity_combine,
            &weapon_combine,
            &invalid_mission_id_list,
            &kpi_config,
            &player_id_to_name,
            &character_id_to_game_id,
            &scout_special_player_set,
        )?;

        debug!("data prepared in {:?}", begin.elapsed());

        let begin = Instant::now();

        let result = generate_bot_kpi_info(
            &cached_mission_list,
            &mission_kpi_cached_info_list,
            &invalid_mission_id_list,
            &watchlist_player_id_list,
            &player_id_to_name,
            &global_kpi_state,
            &kpi_config,
        );

        debug!("bot kpi info generated in {:?}", begin.elapsed());
        Ok(result)
    })
    .await
    .unwrap();

    match result {
        Ok(x) => Json(APIResponse::ok(x)),
        Err(()) => Json(APIResponse::internal_error()),
    }
}
