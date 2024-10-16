use super::{CharacterDamageInfo, CharacterFriendlyFireInfo};
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

#[get("/character")]
async fn get_damage_character(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<HashMap<String, CharacterDamageInfo>>> {
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

        let character_list: Vec<(i16, String)> = match character::table
            .select((character::id, character::character_game_id))
            .load(&mut db_conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get character list from db: {}", e);
                return Err(());
            }
        };

        let character_id_to_game_id = character_list.into_iter().collect::<HashMap<_, _>>();

        let player_list: Vec<(i16, String)> = match player::table
            .select((player::id, player::player_name))
            .load(&mut db_conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get player list from db: {}", e);
                return Err(());
            }
        };

        let player_id_to_name = player_list.into_iter().collect::<HashMap<_, _>>();

        debug!("data prepared in {:?}", begin.elapsed());
        let begin = Instant::now();

        let result = generate(
            &cached_mission_list,
            &invalid_mission_id_list,
            &character_id_to_game_id,
            &character_game_id_to_name,
            &player_id_to_name,
        );

        debug!("character damage info generated in {:?}", begin.elapsed());

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
    character_game_id_to_name: &HashMap<String, String>,
    player_id_to_name: &HashMap<i16, String>,
) -> HashMap<String, CharacterDamageInfo> {
    let player_name_to_id = player_id_to_name
        .iter()
        .map(|(k, v)| (v.clone(), *k))
        .collect::<HashMap<_, _>>();
    let invalid_mission_id_set = invalid_mission_id_list
        .iter()
        .copied()
        .collect::<HashSet<_>>();

    let cached_mission_list = cached_mission_list
        .into_iter()
        .filter(|item| !invalid_mission_id_set.contains(&item.mission_info.id))
        .collect::<Vec<_>>();

    let mut result: HashMap<_, CharacterDamageInfo> = HashMap::new();

    for mission in cached_mission_list {
        let player_id_to_character_id = mission
            .player_info
            .iter()
            .map(|item| {
                (
                    item.player_id,
                    character_id_to_game_id.get(&item.character_id).unwrap(),
                )
            })
            .collect::<HashMap<_, _>>();

        let mut player_ff_take_map: HashMap<i16, f64> =
            HashMap::with_capacity(mission.player_info.len());
        let mut player_ff_cause_map: HashMap<i16, f64> =
            HashMap::with_capacity(mission.player_info.len());

        for (&player_id, player_damage_info) in &mission.damage_info {
            if !mission.player_index.contains_key(&player_id) {
                continue;
            }

            let damage = player_damage_info
                .values()
                .filter(|&item| item.taker_type != 1)
                .map(|item| item.total_amount)
                .sum::<f64>();

            for (taker_game_id, pack) in player_damage_info {
                if pack.taker_type == 1 && pack.taker_id != player_id {
                    let take_player_id = *player_name_to_id.get(taker_game_id).unwrap();

                    let take_entry = player_ff_take_map.entry(take_player_id).or_default();

                    *take_entry += pack.total_amount;

                    let cause_entry = player_ff_cause_map.entry(player_id).or_default();

                    *cause_entry += pack.total_amount;
                }
            }

            let player_index = *mission.player_index.get(&player_id).unwrap();

            let player_character_game_id = *player_id_to_character_id.get(&player_id).unwrap();

            let entry = result
                .entry(player_character_game_id)
                .or_insert(CharacterDamageInfo {
                    damage: 0.0,
                    friendly_fire: CharacterFriendlyFireInfo {
                        cause: 0.0,
                        take: 0.0,
                    },
                    player_index: 0.0,
                    mapped_name: character_game_id_to_name
                        .get(player_character_game_id)
                        .map_or(player_character_game_id.clone(), |x| x.clone()),
                });

            entry.damage += damage;
            entry.player_index += player_index;
        }

        for (player_id, ff_take) in player_ff_take_map {
            let player_character = *player_id_to_character_id.get(&player_id).unwrap();
            let entry = result.get_mut(player_character).unwrap();
            entry.friendly_fire.take += ff_take;
        }

        for (player_id, ff_cause) in player_ff_cause_map {
            let player_character = *player_id_to_character_id.get(&player_id).unwrap();
            let entry = result.get_mut(player_character).unwrap();
            entry.friendly_fire.cause += ff_cause;
        }
    }

    result
        .into_iter()
        .map(|(k, v)| (k.clone(), v))
        .collect::<HashMap<_, _>>()
}
