use super::{DeltaData, GeneralInfo};
use crate::cache::mission::MissionCachedInfo;
use crate::db::schema::*;
use crate::hazard_id_to_real;
use crate::{APIResponse, AppState, DbPool};
use actix_web::{
    get,
    web::{self, Data, Json},
};
use diesel::prelude::*;
use log::{debug, error};
use std::collections::HashSet;
use std::time::Instant;

#[get("/")]
async fn get_general(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<GeneralInfo>> {
    let mapping = app_state.mapping.lock().unwrap();

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

        let watchlist_player_id_list: Vec<i16> = match player::table
            .select(player::id)
            .filter(player::friend.eq(true))
            .load(&mut db_conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get watchlist from db: {}", e);
                return Err(());
            }
        };

        debug!("data prepared in {:?}", begin.elapsed());
        let begin = Instant::now();

        let result = generate(
            &cached_mission_list,
            &invalid_mission_id_list,
            &watchlist_player_id_list,
        );

        debug!("general info generated in {:?}", begin.elapsed());

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
) -> GeneralInfo {
    let game_count = cached_mission_list.len() as i32;

    let invalid_mission_id_set = invalid_mission_id_list
        .iter()
        .copied()
        .collect::<HashSet<_>>();

    let cached_mission_list = cached_mission_list
        .iter()
        .filter(|item| !invalid_mission_id_set.contains(&item.mission_info.id))
        .collect::<Vec<_>>();

    let valid_game_count = cached_mission_list.len();

    let valid_rate = valid_game_count as f64 / game_count as f64;

    let total_total_mission_time = cached_mission_list
        .iter()
        .map(|item| item.mission_info.mission_time as i64)
        .sum::<i64>();
    let prev_count = match valid_game_count * 8 / 10 {
        0..10 => 10,
        x => x,
    };

    let prev_count = if prev_count >= valid_game_count {
        valid_game_count
    } else {
        prev_count
    };

    let prev_mission_list = &cached_mission_list[0..prev_count];
    let recent_mission_list = &cached_mission_list[prev_count..];

    let prev_total_mission_time = prev_mission_list
        .iter()
        .map(|item| item.mission_info.mission_time as u64)
        .sum::<u64>();

    let recent_total_mission_time = recent_mission_list
        .iter()
        .map(|item| item.mission_info.mission_time as u64)
        .sum::<u64>();

    let average_mission_time = DeltaData {
        prev: (prev_total_mission_time as f64 / prev_count as f64) as i16,
        recent: match recent_mission_list.len() {
            0 => (total_total_mission_time as f64 / valid_game_count as f64) as i16,
            _ => (recent_total_mission_time as f64 / recent_mission_list.len() as f64) as i16,
        },
        total: (total_total_mission_time as f64 / valid_game_count as f64) as i16,
    };

    let unique_player_id_set = cached_mission_list
        .iter()
        .map(|item| {
            item.player_info
                .iter()
                .map(|player_info| player_info.player_id)
        })
        .flatten()
        .collect::<HashSet<_>>();

    let unique_player_count = unique_player_id_set.len() as i32;

    let watchlist_player_id_set = watchlist_player_id_list
        .iter()
        .copied()
        .collect::<HashSet<_>>();

    let total_open_room_count = cached_mission_list
        .iter()
        .filter(|item| {
            for player_info in &item.player_info {
                if !watchlist_player_id_set.contains(&player_info.player_id) {
                    return true;
                }
            }

            return false;
        })
        .count();

    let prev_open_room_count = prev_mission_list
        .iter()
        .filter(|item| {
            for player_info in &item.player_info {
                if !watchlist_player_id_set.contains(&player_info.player_id) {
                    return true;
                }
            }

            return false;
        })
        .count();

    let recent_open_room_count = recent_mission_list
        .iter()
        .filter(|item| {
            for player_info in &item.player_info {
                if !watchlist_player_id_set.contains(&player_info.player_id) {
                    return true;
                }
            }

            return false;
        })
        .count();

    let open_room_rate = DeltaData {
        prev: prev_open_room_count as f64 / prev_count as f64,
        recent: match recent_mission_list.len() {
            0 => total_open_room_count as f64 / valid_game_count as f64,
            _ => recent_open_room_count as f64 / recent_mission_list.len() as f64,
        },
        total: total_open_room_count as f64 / valid_game_count as f64,
    };

    let total_pass_count = cached_mission_list
        .iter()
        .filter(|item| item.mission_info.result == 0)
        .count();

    let prev_pass_count = prev_mission_list
        .iter()
        .filter(|item| item.mission_info.result == 0)
        .count();

    let recent_pass_count = recent_mission_list
        .iter()
        .filter(|item| item.mission_info.result == 0)
        .count();

    let pass_rate = DeltaData {
        prev: prev_pass_count as f64 / prev_count as f64,
        recent: match recent_mission_list.len() {
            0 => total_pass_count as f64 / valid_game_count as f64,
            _ => recent_pass_count as f64 / recent_mission_list.len() as f64,
        },
        total: total_pass_count as f64 / valid_game_count as f64,
    };

    let total_difficulty = cached_mission_list
        .iter()
        .map(|item| hazard_id_to_real(item.mission_info.hazard_id))
        .sum::<f64>();

    let prev_difficulty = prev_mission_list
        .iter()
        .map(|item| hazard_id_to_real(item.mission_info.hazard_id))
        .sum::<f64>();

    let recent_difficulty = recent_mission_list
        .iter()
        .map(|item| hazard_id_to_real(item.mission_info.hazard_id))
        .sum::<f64>();

    let average_difficulty = DeltaData {
        prev: prev_difficulty / prev_count as f64,
        recent: match recent_mission_list.len() {
            0 => total_difficulty / valid_game_count as f64,
            _ => recent_difficulty / recent_mission_list.len() as f64,
        },
        total: total_difficulty / valid_game_count as f64,
    };

    let total_kill_num = cached_mission_list
        .iter()
        .map(|item| {
            item.kill_info
                .values()
                .map(|player_data| {
                    player_data
                        .values()
                        .map(|pack| pack.total_amount)
                        .sum::<i32>()
                })
                .sum::<i32>()
        })
        .sum::<i32>();

    let prev_kill_num = prev_mission_list
        .iter()
        .map(|item| {
            item.kill_info
                .values()
                .map(|player_data| {
                    player_data
                        .values()
                        .map(|pack| pack.total_amount)
                        .sum::<i32>()
                })
                .sum::<i32>()
        })
        .sum::<i32>();

    let recent_kill_num = recent_mission_list
        .iter()
        .map(|item| {
            item.kill_info
                .values()
                .map(|player_data| {
                    player_data
                        .values()
                        .map(|pack| pack.total_amount)
                        .sum::<i32>()
                })
                .sum::<i32>()
        })
        .sum::<i32>();

    let average_kill_num = DeltaData {
        prev: (prev_kill_num as f64 / prev_count as f64) as i16,
        recent: match recent_mission_list.len() {
            0 => (total_kill_num as f64 / valid_game_count as f64) as i16,
            _ => (recent_kill_num as f64 / recent_mission_list.len() as f64) as i16,
        },
        total: (total_kill_num as f64 / valid_game_count as f64) as i16,
    };

    let total_damage = cached_mission_list
        .iter()
        .map(|item| {
            item.damage_info
                .values()
                .map(|player_data| {
                    player_data
                        .values()
                        .map(|pack| pack.total_amount)
                        .sum::<f64>()
                })
                .sum::<f64>()
        })
        .sum::<f64>();

    let prev_damage = prev_mission_list
        .iter()
        .map(|item| {
            item.damage_info
                .values()
                .map(|player_data| {
                    player_data
                        .values()
                        .map(|pack| pack.total_amount)
                        .sum::<f64>()
                })
                .sum::<f64>()
        })
        .sum::<f64>();

    let recent_damage = recent_mission_list
        .iter()
        .map(|item| {
            item.damage_info
                .values()
                .map(|player_data| {
                    player_data
                        .values()
                        .map(|pack| pack.total_amount)
                        .sum::<f64>()
                })
                .sum::<f64>()
        })
        .sum::<f64>();

    let average_damage = DeltaData {
        prev: prev_damage / prev_count as f64,
        recent: match recent_mission_list.len() {
            0 => total_damage / valid_game_count as f64,
            _ => recent_damage / recent_mission_list.len() as f64,
        },
        total: total_damage / valid_game_count as f64,
    };

    let total_avergae_death_num_per_player = cached_mission_list
        .iter()
        .map(|item| &item.player_info)
        .map(|player_info_list| {
            player_info_list
                .iter()
                .map(|player_info| player_info.death_num as f64)
                .sum::<f64>()
                / player_info_list.len() as f64
        })
        .sum::<f64>();

    let prev_average_death_num_per_player = prev_mission_list
        .iter()
        .map(|item| &item.player_info)
        .map(|player_info_list| {
            player_info_list
                .iter()
                .map(|player_info| player_info.death_num as f64)
                .sum::<f64>()
                / player_info_list.len() as f64
        })
        .sum::<f64>();

    let recent_average_death_num_per_player = recent_mission_list
        .iter()
        .map(|item| &item.player_info)
        .map(|player_info_list| {
            player_info_list
                .iter()
                .map(|player_info| player_info.death_num as f64)
                .sum::<f64>()
                / player_info_list.len() as f64
        })
        .sum::<f64>();

    let average_death_num_per_player = DeltaData {
        prev: prev_average_death_num_per_player / prev_count as f64,
        recent: match recent_mission_list.len() {
            0 => total_avergae_death_num_per_player / valid_game_count as f64,
            _ => recent_average_death_num_per_player / recent_mission_list.len() as f64,
        },
        total: total_avergae_death_num_per_player / valid_game_count as f64,
    };

    let total_minerals_mined = cached_mission_list
        .iter()
        .map(|item| {
            item.resource_info
                .values()
                .map(|player_resource_info| player_resource_info.values().sum::<f64>())
                .sum::<f64>()
        })
        .sum::<f64>();

    let prev_minerals_mined = prev_mission_list
        .iter()
        .map(|item| {
            item.resource_info
                .values()
                .map(|player_resource_info| player_resource_info.values().sum::<f64>())
                .sum::<f64>()
        })
        .sum::<f64>();

    let recent_minerals_mined = recent_mission_list
        .iter()
        .map(|item| {
            item.resource_info
                .values()
                .map(|player_resource_info| player_resource_info.values().sum::<f64>())
                .sum::<f64>()
        })
        .sum::<f64>();

    let average_minerals_mined = DeltaData {
        prev: prev_minerals_mined / prev_count as f64,
        recent: match recent_mission_list.len() {
            0 => total_minerals_mined / valid_game_count as f64,
            _ => recent_minerals_mined / recent_mission_list.len() as f64,
        },
        total: total_minerals_mined / valid_game_count as f64,
    };

    let total_supply_count_per_player = cached_mission_list
        .iter()
        .map(|item| {
            item.supply_info
                .values()
                .map(|player_supply_list| player_supply_list.len() as f64)
                .sum::<f64>()
                / item.player_info.len() as f64
        })
        .sum::<f64>();

    let prev_supply_count_per_player = prev_mission_list
        .iter()
        .map(|item| {
            item.supply_info
                .values()
                .map(|player_supply_list| player_supply_list.len() as f64)
                .sum::<f64>()
                / item.player_info.len() as f64
        })
        .sum::<f64>();

    let recent_supply_count_per_player = recent_mission_list
        .iter()
        .map(|item| {
            item.supply_info
                .values()
                .map(|player_supply_list| player_supply_list.len() as f64)
                .sum::<f64>()
                / item.player_info.len() as f64
        })
        .sum::<f64>();

    let average_supply_count_per_player = DeltaData {
        prev: prev_supply_count_per_player / prev_count as f64,
        recent: match recent_mission_list.len() {
            0 => total_supply_count_per_player / valid_game_count as f64,
            _ => recent_supply_count_per_player / recent_mission_list.len() as f64,
        },
        total: total_supply_count_per_player / valid_game_count as f64,
    };

    let total_reward_credit = cached_mission_list
        .iter()
        .map(|item| item.mission_info.reward_credit)
        .sum::<f64>();

    let prev_reward_credit = prev_mission_list
        .iter()
        .map(|item| item.mission_info.reward_credit)
        .sum::<f64>();

    let recent_reward_credit = recent_mission_list
        .iter()
        .map(|item| item.mission_info.reward_credit)
        .sum::<f64>();

    let average_reward_credit = DeltaData {
        prev: prev_reward_credit / prev_count as f64,
        recent: match recent_mission_list.len() {
            0 => total_reward_credit / valid_game_count as f64,
            _ => recent_reward_credit / recent_mission_list.len() as f64,
        },
        total: total_reward_credit / valid_game_count as f64,
    };

    GeneralInfo {
        game_count,
        valid_rate,
        total_mission_time: total_total_mission_time,
        average_mission_time,
        unique_player_count,
        open_room_rate,
        pass_rate,
        average_difficulty,
        average_kill_num,
        average_damage,
        average_death_num_per_player,
        average_minerals_mined,
        average_supply_count_per_player,
        average_reward_credit,
    }
}
