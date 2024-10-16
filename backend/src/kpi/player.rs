use crate::cache::kpi::*;
use crate::cache::mission::*;
use crate::db::models::*;
use crate::db::schema::*;
use crate::kpi::KPIConfig;
use crate::mission::mission::generate_mission_kpi;
use crate::mission::MissionKPIInfo;
use crate::{APIResponse, AppState, DbPool};
use actix_web::{
    get,
    web::{self, Data, Json},
};
use diesel::prelude::*;
use log::{debug, error};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[derive(Serialize, Clone, Copy)]
pub struct PlayerMissionKPIInfo {
    #[serde(rename = "missionId")]
    pub mission_id: i32,
    #[serde(rename = "beginTimestamp")]
    pub begin_timestamp: i64,
    #[serde(rename = "playerIndex")]
    pub player_index: f64,
    #[serde(rename = "missionKPI")]
    pub mission_kpi: f64,
}

#[derive(Serialize, Clone)]
pub struct PlayerCharacterKPIInfo {
    #[serde(rename = "playerIndex")]
    pub player_index: f64,
    #[serde(rename = "characterKPI")]
    pub character_kpi: f64,
    #[serde(rename = "characterKPIType")]
    pub character_kpi_type: String,
    #[serde(rename = "missionList")]
    pub mission_list: Vec<PlayerMissionKPIInfo>,
}

#[derive(Serialize)]
pub struct PlayerKPIInfo {
    #[serde(rename = "playerIndex")]
    pub player_index: f64,
    #[serde(rename = "playerKPI")]
    pub player_kpi: f64,
    #[serde(rename = "byCharacter")]
    pub by_character: HashMap<String, PlayerCharacterKPIInfo>,
}

pub fn generate_player_kpi(
    cached_mission_list: &[MissionCachedInfo],
    mission_kpi_cached_info_list: &[MissionKPICachedInfo],
    invalid_mission_id_list: &[i32],
    watchlist_player_id_list: &[i16],
    player_id_to_name: &HashMap<i16, String>,
    global_kpi_state: &CachedGlobalKPIState,
    kpi_config: &KPIConfig,
) -> HashMap<String, PlayerKPIInfo> {
    let player_name_to_id = player_id_to_name
        .iter()
        .map(|(id, name)| (name, *id))
        .collect::<HashMap<_, _>>();

    let watchlist_player_name_set = watchlist_player_id_list
        .iter()
        .map(|id| player_id_to_name.get(id).unwrap())
        .collect::<HashSet<_>>();

    let invalid_mission_id_set = invalid_mission_id_list
        .iter()
        .copied()
        .collect::<HashSet<_>>();

    let mission_kpi_cached_info_list = mission_kpi_cached_info_list
        .into_iter()
        .filter(|item| !invalid_mission_id_set.contains(&item.mission_id))
        .collect::<Vec<_>>();

    let mission_id_to_cached_info = cached_mission_list
        .iter()
        .map(|mission_info| (mission_info.mission_info.id, mission_info))
        .collect::<HashMap<_, _>>();

    let mission_kpi_by_mission_id = mission_kpi_cached_info_list
        .iter()
        .map(|mission_kpi_info| {
            (
                mission_kpi_info.mission_id,
                (
                    mission_kpi_info.mission_id,
                    generate_mission_kpi(
                        &mission_kpi_info,
                        player_id_to_name,
                        global_kpi_state,
                        kpi_config,
                    ),
                ),
            )
        })
        .collect::<HashMap<_, _>>();

    let mut player_name_to_character_type_to_mission_list: HashMap<
        &String,
        HashMap<&String, Vec<(i32, &MissionKPIInfo)>>,
    > = HashMap::new();

    for (mission_id, mission_kpi_info_list) in mission_kpi_by_mission_id.values() {
        for mission_kpi_info in mission_kpi_info_list {
            let player_name = &mission_kpi_info.player_name;
            let character_type = &mission_kpi_info.kpi_character_type;

            player_name_to_character_type_to_mission_list
                .entry(player_name)
                .or_default()
                .entry(character_type)
                .or_default()
                .push((*mission_id, mission_kpi_info));
        }
    }

    let mut result = HashMap::new();

    for (player_name, character_type_to_mission_list) in
        player_name_to_character_type_to_mission_list
    {
        if !watchlist_player_name_set.contains(player_name) {
            continue;
        }
        let mut total_player_player_index = 0.0;
        let mut player_kpi_weighted_sum = 0.0;

        let mut by_character = HashMap::new();
        for (character_type, mission_list) in character_type_to_mission_list {
            let mut total_character_player_index = 0.0;
            let mut mission_kpi_weighted_sum = 0.0;

            let mut result_mission_list = Vec::new();

            for (mission_id, mission_kpi_info) in mission_list {
                let mission_info = *mission_id_to_cached_info.get(&mission_id).unwrap();
                let player_index = *mission_info
                    .player_index
                    .get(player_name_to_id.get(player_name).unwrap())
                    .unwrap();
                result_mission_list.push(PlayerMissionKPIInfo {
                    mission_id,
                    begin_timestamp: mission_info.mission_info.begin_timestamp,
                    player_index,
                    mission_kpi: mission_kpi_info.mission_kpi,
                });

                total_character_player_index += player_index;
                mission_kpi_weighted_sum += player_index * mission_kpi_info.mission_kpi;

                total_player_player_index += player_index;
                player_kpi_weighted_sum += player_index * mission_kpi_info.mission_kpi;
            }

            let player_character_kpi_info = PlayerCharacterKPIInfo {
                player_index: total_character_player_index,
                character_kpi: mission_kpi_weighted_sum / total_character_player_index,
                character_kpi_type: character_type.to_string(),
                mission_list: result_mission_list,
            };

            by_character.insert(character_type.to_string(), player_character_kpi_info);
        }

        let player_kpi_info = PlayerKPIInfo {
            player_index: total_player_player_index,
            player_kpi: player_kpi_weighted_sum / total_player_player_index,
            by_character,
        };

        result.insert(player_name.clone(), player_kpi_info);
    }

    result
}

#[get("/player_kpi")]
async fn get_player_kpi(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<HashMap<String, PlayerKPIInfo>>> {
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

        let result = generate_player_kpi(
            &cached_mission_list,
            &mission_kpi_cached_info_list,
            &invalid_mission_id_list,
            &watchlist_player_id_list,
            &player_id_to_name,
            &global_kpi_state,
            &kpi_config,
        );

        debug!("player kpi generated in {:?}", begin.elapsed());
        Ok(result)
    })
    .await
    .unwrap();

    match result {
        Ok(x) => Json(APIResponse::ok(x)),
        Err(()) => Json(APIResponse::internal_error()),
    }
}
