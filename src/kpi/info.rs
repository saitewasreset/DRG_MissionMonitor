use super::APIWeightTableData;
use crate::cache::kpi::CachedGlobalKPIState;
use crate::db::models::*;
use crate::db::schema::*;
use crate::kpi::CharacterKPIType;
use crate::kpi::IndexTransformRange;
use crate::{APIResponse, AppState, DbPool};
use actix_web::{
    get,
    web::{self, Data, Json},
};
use diesel::prelude::*;
use log::error;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct GammaInnerInfo {
    #[serde(rename = "playerIndex")]
    pub player_index: f64,
    pub value: f64,
    pub ratio: f64,
}

#[get("/gamma")]
async fn get_gamma_info(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<HashMap<String, HashMap<String, GammaInnerInfo>>>> {
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

        let result = CachedGlobalKPIState::from_redis_all(
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
        )?;

        Ok(result)
    })
    .await
    .unwrap();

    match result {
        Ok(x) => {
            let mut result: HashMap<String, HashMap<String, GammaInnerInfo>> = HashMap::new();
            for (character_kpi_type, character_component) in x.character_correction_factor {
                for (kpi_component, character_data) in character_component {
                    result
                        .entry(kpi_component.to_string())
                        .or_default()
                        .entry(character_kpi_type.to_string())
                        .or_insert(GammaInnerInfo {
                            player_index: character_data.player_index,
                            value: character_data.value,
                            ratio: character_data.correction_factor,
                        });
                }
            }

            Json(APIResponse::ok(result))
        }
        Err(()) => Json(APIResponse::internal_error()),
    }
}

#[get("/transform_range_info")]
async fn get_transform_range_info(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<HashMap<String, HashMap<String, Vec<IndexTransformRange>>>>> {
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

        let result = CachedGlobalKPIState::get_cached(
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

        Ok(result)
    })
    .await
    .unwrap();

    match result {
        Ok(x) => Json(APIResponse::ok(
            x.transform_range
                .iter()
                .map(|(character_kpi_type, character_info)| {
                    (
                        character_kpi_type.to_string(),
                        character_info
                            .iter()
                            .map(|(character_id, info)| (character_id.to_string(), info.clone()))
                            .collect(),
                    )
                })
                .collect::<HashMap<_, _>>(),
        )),
        Err(()) => Json(APIResponse::internal_error()),
    }
}

#[get("/weight_table")]
async fn get_weight_table(app_state: Data<AppState>) -> Json<APIResponse<Vec<APIWeightTableData>>> {
    let entity_game_id_to_name = app_state.mapping.lock().unwrap().entity_mapping.clone();

    let kpi_config = match app_state.kpi_config.lock().unwrap().clone() {
        Some(x) => x,
        None => {
            return Json(APIResponse::config_required("kpi_config"));
        }
    };

    let mut result = Vec::new();

    for entity_game_id in entity_game_id_to_name.keys() {
        let priority = *kpi_config
            .priority_table
            .get(entity_game_id)
            .unwrap_or(&0.0);

        let driller = *kpi_config
            .character_weight_table
            .get(&CharacterKPIType::Driller)
            .unwrap_or(&HashMap::new())
            .get(entity_game_id)
            .unwrap_or(&1.0);

        let gunner = *kpi_config
            .character_weight_table
            .get(&CharacterKPIType::Gunner)
            .unwrap_or(&HashMap::new())
            .get(entity_game_id)
            .unwrap_or(&1.0);

        let engineer = *kpi_config
            .character_weight_table
            .get(&CharacterKPIType::Engineer)
            .unwrap_or(&HashMap::new())
            .get(entity_game_id)
            .unwrap_or(&1.0);

        let scout = *kpi_config
            .character_weight_table
            .get(&CharacterKPIType::Scout)
            .unwrap_or(&HashMap::new())
            .get(entity_game_id)
            .unwrap_or(&1.0);

        let scout_special = *kpi_config
            .character_weight_table
            .get(&CharacterKPIType::ScoutSpecial)
            .unwrap_or(&HashMap::new())
            .get(entity_game_id)
            .unwrap_or(&1.0);

        result.push(APIWeightTableData {
            entity_game_id: entity_game_id.clone(),
            priority,
            driller,
            gunner,
            engineer,
            scout,
            scout_special,
        });
    }

    Json(APIResponse::ok(result))
}
