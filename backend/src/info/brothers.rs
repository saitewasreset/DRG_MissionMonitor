use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::cache::mission::MissionCachedInfo;
use crate::RE_SPOT_TIME_THRESHOLD;
use crate::{APIResponse, AppState, DbPool};
use actix_web::{
    get,
    web::{self, Data, Json},
};

use crate::db::models::*;
use crate::db::schema::*;
use diesel::prelude::*;
use log::{debug, error};
use std::time::Instant;

#[derive(Serialize)]
pub struct OverallInfo {
    // 平均游戏局数
    #[serde(rename = "playerAverageSpot")]
    pub player_average_spot: f64,
    // 路人玩家数
    #[serde(rename = "unfamiliarPlayerCount")]
    pub unfamiliar_player_count: i32,
    // 再相遇概率
    #[serde(rename = "playerGeTwoPercent")]
    pub player_ge_two_percent: f64,
    //多于一局概率
    #[serde(rename = "playerSpotPercent")]
    pub player_spot_percent: f64,
}

#[derive(Serialize)]
pub struct PlayerInfo {
    #[serde(rename = "gameCount")]
    pub game_count: i32,
    #[serde(rename = "lastSpot")]
    pub last_spot: i64,
    #[serde(rename = "presenceTime")]
    pub presence_time: i32,
    // 再相遇次数
    #[serde(rename = "spotCount")]
    pub spot_count: i32,
    #[serde(rename = "timestampList")]
    pub timestamp_list: Vec<i64>,
}

#[derive(Serialize)]
pub struct APIBrothers {
    pub overall: OverallInfo,
    pub player: HashMap<String, PlayerInfo>,
}

fn generate(
    cached_mission_list: &[MissionCachedInfo],
    player_id_to_name: &HashMap<i16, String>,
    watchlist_player_id_list: &[i16],
) -> APIBrothers {
    let watchlist_player_id_set = watchlist_player_id_list
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    let mut player_map = HashMap::new();

    for mission in cached_mission_list {
        for player_info in &mission.player_info {
            if watchlist_player_id_set.contains(&player_info.player_id) {
                continue;
            }
            let player_entry = player_map
                .entry(player_info.player_id)
                .or_insert(PlayerInfo {
                    game_count: 0,
                    last_spot: 0,
                    presence_time: 0,
                    spot_count: 0,
                    timestamp_list: Vec::new(),
                });

            player_entry.game_count += 1;
            if mission.mission_info.begin_timestamp > player_entry.last_spot {
                player_entry.last_spot = mission.mission_info.begin_timestamp;
            }

            player_entry.presence_time += player_info.present_time as i32;

            player_entry
                .timestamp_list
                .push(mission.mission_info.begin_timestamp);
        }
    }

    for (_, player_info) in player_map.iter_mut() {
        player_info.timestamp_list.sort_unstable();
        let mut last_timestamp = player_info.timestamp_list[0];
        for &timestamp in &player_info.timestamp_list {
            if timestamp - last_timestamp > RE_SPOT_TIME_THRESHOLD {
                player_info.spot_count += 1;
            }
            last_timestamp = timestamp;
        }
    }

    let player_count = player_map.len() as i32;
    let total_spot_count = player_map.values().map(|x| x.spot_count).sum::<i32>();

    let player_average_spot = total_spot_count as f64 / player_map.len() as f64;

    let player_ge_two_count = player_map.values().filter(|x| x.game_count >= 2).count();
    let player_ge_two_percent = player_ge_two_count as f64 / player_count as f64;

    let player_spot_count = player_map.values().filter(|x| x.spot_count >= 1).count();
    let player_spot_percent = player_spot_count as f64 / player_count as f64;

    APIBrothers {
        overall: OverallInfo {
            player_average_spot,
            unfamiliar_player_count: player_count,
            player_ge_two_percent,
            player_spot_percent,
        },
        player: player_map
            .into_iter()
            .map(|(player_id, player_info)| {
                (
                    player_id_to_name.get(&player_id).unwrap().clone(),
                    player_info,
                )
            })
            .collect(),
    }
}

#[get("/brothers")]
async fn get_brothers_info(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<APIBrothers>> {
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

        let watchlist_player_id_list: Vec<i16> = match player::table
            .select(player::id)
            .filter(player::friend.eq(true))
            .load::<i16>(&mut db_conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get player id list: {}", e);
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

        debug!("data prepared in {:?}", begin.elapsed());
        let begin = Instant::now();

        let result = generate(
            &cached_mission_list,
            &player_id_to_name,
            &watchlist_player_id_list,
        );

        debug!("brothers info generated in {:?}", begin.elapsed());

        Ok(result)
    })
    .await
    .unwrap();

    match result {
        Ok(x) => Json(APIResponse::ok(x)),
        Err(()) => Json(APIResponse::internal_error()),
    }
}
