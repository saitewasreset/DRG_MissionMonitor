use super::{PlayerData, PlayerInfo};
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

#[get("/player")]
async fn get_player(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<PlayerInfo>> {
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

        let player_list = match player::table.select(Player::as_select()).load(&mut db_conn) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get player list from db: {}", e);
                return Err(());
            }
        };

        let watchlist_player_id_list: Vec<i16> = player_list
            .iter()
            .filter(|x| x.friend)
            .map(|x| x.id)
            .collect();

        let player_id_to_name = player_list
            .into_iter()
            .map(|x| (x.id, x.player_name))
            .collect();

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
            &watchlist_player_id_list,
            &player_id_to_name,
            &character_id_to_game_id,
            character_game_id_to_name,
        );

        debug!("player info generated in {:?}", begin.elapsed());

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
    watchlist_player_id_list: &[i16],
    player_id_to_name: &HashMap<i16, String>,
    character_id_to_game_id: &HashMap<i16, String>,
    character_game_id_to_name: HashMap<String, String>,
) -> PlayerInfo {
    let invalid_mission_id_set = invalid_mission_id_list
        .iter()
        .copied()
        .collect::<HashSet<_>>();

    let watchlist_player_id_set = watchlist_player_id_list
        .iter()
        .copied()
        .collect::<HashSet<_>>();

    let cached_mission_list = cached_mission_list
        .into_iter()
        .filter(|item| !invalid_mission_id_set.contains(&item.mission_info.id))
        .collect::<Vec<_>>();

    let mut mission_list_by_player: HashMap<i16, Vec<&MissionCachedInfo>> = HashMap::new();

    for mission in cached_mission_list {
        for player_info in &mission.player_info {
            if !watchlist_player_id_set.contains(&player_info.player_id) {
                continue;
            }
            mission_list_by_player
                .entry(player_info.player_id)
                .or_insert_with(Vec::new)
                .push(mission);
        }
    }

    let mut overall_player_data_map = HashMap::with_capacity(mission_list_by_player.len());
    let mut prev_player_data_map = HashMap::with_capacity(mission_list_by_player.len());

    for (player_id, player_mission_list) in mission_list_by_player {
        let prev_count = match player_mission_list.len() * 8 / 10 {
            0..10 => 10,
            x => x,
        };

        let prev_count = if prev_count > player_mission_list.len() {
            player_mission_list.len()
        } else {
            prev_count
        };

        let prev_mission_list = &player_mission_list[0..prev_count];

        let overall_data =
            generate_for_player(&player_mission_list[..], character_id_to_game_id, player_id);
        let prev_data = generate_for_player(prev_mission_list, character_id_to_game_id, player_id);

        let player_name = player_id_to_name.get(&player_id).unwrap();

        overall_player_data_map.insert(player_name.clone(), overall_data);
        prev_player_data_map.insert(player_name.clone(), prev_data);
    }

    PlayerInfo {
        character_map: character_game_id_to_name,
        player_data: overall_player_data_map,
        prev_player_data: prev_player_data_map,
    }
}

fn generate_for_player(
    player_mission_list: &[&MissionCachedInfo],
    character_id_to_game_id: &HashMap<i16, String>,
    player_id: i16,
) -> PlayerData {
    let average_death_num = player_mission_list
        .iter()
        .map(|item| {
            for player_info in &item.player_info {
                if player_info.player_id == player_id {
                    return player_info.death_num as i32;
                }
            }
            unreachable!();
        })
        .sum::<i32>() as f64
        / player_mission_list.len() as f64;

    let average_minerals_mined = player_mission_list
        .iter()
        .map(|item| match item.resource_info.get(&player_id) {
            Some(info) => info.values().sum::<f64>(),
            None => 0.0,
        })
        .sum::<f64>()
        / player_mission_list.len() as f64;

    let average_revive_num = player_mission_list
        .iter()
        .map(|item| {
            for player_info in &item.player_info {
                if player_info.player_id == player_id {
                    return player_info.revive_num as i32;
                }
            }
            unreachable!();
        })
        .sum::<i32>() as f64
        / player_mission_list.len() as f64;

    let average_supply_count = player_mission_list
        .iter()
        .map(|item| match item.supply_info.get(&player_id) {
            Some(info) => info.len(),
            None => 0,
        })
        .sum::<usize>() as f64
        / player_mission_list.len() as f64;

    let supply_efficiency_list: Vec<f64> = player_mission_list
        .iter()
        .map(|item| item.supply_info.get(&player_id).into_iter().flatten())
        .flatten()
        .map(|x| x.ammo)
        .collect();

    let average_supply_efficiency =
        2.0 * supply_efficiency_list.iter().sum::<f64>() / supply_efficiency_list.len() as f64;

    let mut character_info: HashMap<&String, i32> = HashMap::new();

    for mission in player_mission_list {
        for player_info in &mission.player_info {
            if player_info.player_id == player_id {
                let entry = character_info
                    .entry(
                        character_id_to_game_id
                            .get(&player_info.character_id)
                            .unwrap(),
                    )
                    .or_default();

                *entry += 1;
            }
        }
    }

    PlayerData {
        average_death_num,
        average_minerals_mined,
        average_revive_num,
        average_supply_count,
        average_supply_efficiency,
        character_info: character_info
            .into_iter()
            .map(|(k, v)| (k.clone(), v))
            .collect(),
        valid_mission_count: player_mission_list.len() as i32,
    }
}
