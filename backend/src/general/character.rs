use super::{CharacterChoiceInfo, CharacterGeneralData, CharacterGeneralInfo};
use crate::cache::mission::MissionCachedInfo;
use crate::db::models::*;
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

#[get("/character")]
async fn get_character_general_info(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<CharacterGeneralInfo>> {
    let mapping = app_state.mapping.lock().unwrap();

    let entity_blacklist_set = mapping.entity_blacklist_set.clone();
    let entity_combine = mapping.entity_combine.clone();
    let weapon_combine = mapping.weapon_combine.clone();
    let character_game_id_to_name = mapping.character_mapping.clone();
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
            .map(|x| (x.id, x.character_game_id))
            .collect::<HashMap<_, _>>();

        debug!("data prepared in {:?}", begin.elapsed());
        let begin = Instant::now();

        let result = generate(
            &cached_mission_list,
            &invalid_mission_id_list,
            &character_id_to_game_id,
            character_game_id_to_name,
        );

        debug!("character general info generated in {:?}", begin.elapsed());

        Ok(result)
    })
    .await
    .unwrap();

    match result {
        Ok(x) => Json(APIResponse::ok(x)),
        Err(()) => Json(APIResponse::internal_error()),
    }
}

#[get("/character_info")]
async fn get_character_choice_info(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<CharacterChoiceInfo>> {
    let mapping = app_state.mapping.lock().unwrap();

    let entity_blacklist_set = mapping.entity_blacklist_set.clone();
    let entity_combine = mapping.entity_combine.clone();
    let weapon_combine = mapping.weapon_combine.clone();
    let character_game_id_to_name = mapping.character_mapping.clone();
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
            .map(|x| (x.id, x.character_game_id))
            .collect::<HashMap<_, _>>();

        debug!("data prepared in {:?}", begin.elapsed());
        let begin = Instant::now();

        let result = generate_choice_info(
            &cached_mission_list,
            &invalid_mission_id_list,
            &character_id_to_game_id,
            character_game_id_to_name,
        );

        debug!("character choice info generated in {:?}", begin.elapsed());

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
    character_id_to_game_id: &HashMap<i16, String>,
    character_game_id_to_name: HashMap<String, String>,
) -> CharacterGeneralInfo {
    let invalid_mission_id_set = invalid_mission_id_list
        .iter()
        .copied()
        .collect::<HashSet<_>>();

    let cached_mission_list = cached_mission_list
        .into_iter()
        .filter(|info| !invalid_mission_id_set.contains(&info.mission_info.id))
        .collect::<Vec<_>>();

    let mut player_index_list_by_character: HashMap<&String, Vec<f64>> = HashMap::new();
    let mut revive_num_list_by_character: HashMap<&String, Vec<i32>> = HashMap::new();
    let mut death_num_list_by_character: HashMap<&String, Vec<i32>> = HashMap::new();
    let mut minerals_mined_list_by_character: HashMap<&String, Vec<f64>> = HashMap::new();
    let mut supply_count_list_by_character: HashMap<&String, Vec<i32>> = HashMap::new();
    let mut supply_efficiency_list_by_character: HashMap<&String, Vec<f64>> = HashMap::new();

    for mission in cached_mission_list {
        for player_info in &mission.player_info {
            let character_game_id = character_id_to_game_id
                .get(&player_info.character_id)
                .unwrap();

            player_index_list_by_character
                .entry(character_game_id)
                .or_insert_with(Vec::new)
                .push(
                    mission
                        .player_index
                        .get(&player_info.player_id)
                        .map(|x| *x)
                        .unwrap_or(0.0),
                );
            revive_num_list_by_character
                .entry(character_game_id)
                .or_insert_with(Vec::new)
                .push(player_info.revive_num as i32);
            death_num_list_by_character
                .entry(character_game_id)
                .or_insert_with(Vec::new)
                .push(player_info.death_num as i32);
            minerals_mined_list_by_character
                .entry(character_game_id)
                .or_insert_with(Vec::new)
                .push(match mission.resource_info.get(&player_info.player_id) {
                    Some(x) => x.values().sum::<f64>(),
                    None => 0.0,
                });
            supply_count_list_by_character
                .entry(character_game_id)
                .or_insert_with(Vec::new)
                .push(match mission.supply_info.get(&player_info.player_id) {
                    Some(x) => x.len() as i32,
                    None => 0,
                });

            let player_supply_efficiency_list = mission
                .supply_info
                .get(&player_info.player_id)
                .into_iter()
                .flatten()
                .map(|x| 2.0 * x.ammo)
                .collect::<Vec<_>>();

            supply_efficiency_list_by_character
                .entry(character_game_id)
                .or_insert_with(Vec::new)
                .extend(player_supply_efficiency_list);
        }
    }

    let mut character_data = HashMap::new();

    for &character_game_id in player_index_list_by_character.keys() {
        character_data.insert(
            character_game_id.clone(),
            CharacterGeneralData {
                player_index: player_index_list_by_character[character_game_id]
                    .iter()
                    .sum::<f64>(),
                revive_num: revive_num_list_by_character[character_game_id]
                    .iter()
                    .sum::<i32>() as f64
                    / revive_num_list_by_character[character_game_id].len() as f64,
                death_num: death_num_list_by_character[character_game_id]
                    .iter()
                    .sum::<i32>() as f64
                    / death_num_list_by_character[character_game_id].len() as f64,
                minerals_mined: minerals_mined_list_by_character[character_game_id]
                    .iter()
                    .sum::<f64>()
                    / minerals_mined_list_by_character[character_game_id].len() as f64,
                supply_count: supply_count_list_by_character[character_game_id]
                    .iter()
                    .sum::<i32>() as f64
                    / supply_count_list_by_character[character_game_id].len() as f64,
                supply_efficiency: supply_efficiency_list_by_character[character_game_id]
                    .iter()
                    .sum::<f64>()
                    / supply_efficiency_list_by_character[character_game_id].len() as f64,
            },
        );
    }

    CharacterGeneralInfo {
        character_data,
        character_mapping: character_game_id_to_name,
    }
}

fn generate_choice_info(
    cached_mission_list: &[MissionCachedInfo],
    invalid_mission_id_list: &[i32],
    character_id_to_game_id: &HashMap<i16, String>,
    character_game_id_to_name: HashMap<String, String>,
) -> CharacterChoiceInfo {
    let invalid_mission_id_set = invalid_mission_id_list
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    let cached_mission_list = cached_mission_list
        .into_iter()
        .filter(|info| !invalid_mission_id_set.contains(&info.mission_info.id))
        .collect::<Vec<_>>();

    let mut character_choice_count: HashMap<String, i32> = HashMap::new();

    for mission in cached_mission_list {
        for player_info in &mission.player_info {
            let character_game_id = character_id_to_game_id
                .get(&player_info.character_id)
                .unwrap();

            *character_choice_count
                .entry(character_game_id.clone())
                .or_default() += 1;
        }
    }

    CharacterChoiceInfo {
        character_choice_count,
        character_mapping: character_game_id_to_name,
    }
}
