use std::collections::HashMap;

use super::{
    MissionDamageInfo, MissionGeneralData, MissionGeneralInfo, MissionGeneralPlayerInfo,
    MissionKPIComponent, MissionKPIInfo, MissionResourceInfo, MissionWeaponDamageInfo,
    PlayerDamageInfo, PlayerFriendlyFireInfo, PlayerResourceData,
};
use crate::cache::kpi::CachedGlobalKPIState;
use crate::cache::mission::{MissionCachedInfo, MissionKPICachedInfo};
use crate::db::models::*;
use crate::kpi::{KPIComponent, KPIConfig};
use crate::{CORRECTION_ITEMS, NITRA_GAME_ID};

use crate::db::schema::*;
use crate::{APIResponse, AppState, DbPool};
use actix_web::{
    get,
    web::{self, Data, Json},
};
use diesel::prelude::*;
use log::{debug, error};
use std::time::Instant;

fn generate_mission_general_info(
    cached_mission_list: &[MissionCachedInfo],
    invalid_mission_list: &[MissionInvalid],
    mission_id: i32,
) -> Option<MissionGeneralInfo> {
    let mut mission_invalid = None;

    for invalid_mission in invalid_mission_list {
        if invalid_mission.mission_id == mission_id {
            mission_invalid = Some(invalid_mission);
            break;
        }
    }

    for mission in cached_mission_list {
        if mission.mission_info.id == mission_id {
            return Some(MissionGeneralInfo {
                mission_id,
                mission_begin_timestamp: mission.mission_info.begin_timestamp,
                mission_invalid: mission_invalid.is_some(),
                mission_invalid_reason: mission_invalid.map_or_else(
                    || "".to_string(),
                    |invalid_mission| invalid_mission.reason.clone(),
                ),
            });
        }
    }

    return None;
}

fn generate_mission_player_character(
    cached_mission_list: &[MissionCachedInfo],
    player_id_to_name: &HashMap<i16, String>,
    character_id_to_game_id: &HashMap<i16, String>,
    mission_id: i32,
) -> Option<HashMap<String, String>> {
    for mission in cached_mission_list {
        if mission.mission_info.id == mission_id {
            let mut result = HashMap::new();
            for player_info in &mission.player_info {
                let character_game_id = character_id_to_game_id
                    .get(&player_info.character_id)
                    .unwrap();
                let player_name = player_id_to_name.get(&player_info.player_id).unwrap();
                result.insert(player_name.clone(), character_game_id.clone());
            }
            return Some(result);
        }
    }

    return None;
}

fn generate_mission_general(
    cached_mission_list: &[MissionCachedInfo],
    player_id_to_name: &HashMap<i16, String>,
    character_id_to_game_id: &HashMap<i16, String>,
    mission_type_id_to_game_id: &HashMap<i16, String>,
    mission_id: i32,
) -> Option<MissionGeneralData> {
    let target_mission = cached_mission_list
        .iter()
        .find(|mission| mission.mission_info.id == mission_id)?;

    let mut mission_player_info = HashMap::with_capacity(target_mission.player_info.len());

    for player_info in &target_mission.player_info {
        let character_game_id = character_id_to_game_id
            .get(&player_info.character_id)
            .unwrap();
        let player_name = player_id_to_name.get(&player_info.player_id).unwrap();
        mission_player_info.insert(
            player_name.clone(),
            MissionGeneralPlayerInfo {
                character_game_id: character_game_id.clone(),
                player_rank: player_info.player_rank,
                character_rank: player_info.character_rank,
                character_promotion: player_info.character_promotion,
                present_time: player_info.present_time,
                revive_num: player_info.revive_num,
                death_num: player_info.death_num,
                player_escaped: player_info.player_escaped,
            },
        );
    }

    let mission_type_game_id = mission_type_id_to_game_id
        .get(&target_mission.mission_info.mission_type_id)
        .unwrap();

    let total_damage = target_mission
        .damage_info
        .values()
        .map(|player_damage_data| player_damage_data.values())
        .flatten()
        .filter(|pack| pack.taker_type != 1)
        .map(|pack| pack.total_amount)
        .sum::<f64>();

    let total_kill = target_mission
        .kill_info
        .values()
        .map(|player_kill_map| player_kill_map.values())
        .flatten()
        .map(|pack| pack.total_amount)
        .sum::<i32>();

    let total_nitra = target_mission
        .resource_info
        .values()
        .map(|player_data| player_data.get(NITRA_GAME_ID))
        .flatten()
        .copied()
        .sum::<f64>();

    let total_minerals = target_mission
        .resource_info
        .values()
        .map(|player_data| player_data.values())
        .flatten()
        .sum::<f64>();

    let total_supply_count = target_mission
        .supply_info
        .values()
        .map(|v| v.len() as i16)
        .sum::<i16>();

    Some(MissionGeneralData {
        begin_timestamp: target_mission.mission_info.begin_timestamp,
        hazard_id: target_mission.mission_info.hazard_id,
        mission_result: target_mission.mission_info.result,
        mission_time: target_mission.mission_info.mission_time,
        mission_type_id: mission_type_game_id.clone(),
        player_info: mission_player_info,
        reward_credit: target_mission.mission_info.reward_credit,
        total_damage,
        total_kill,
        total_minerals,
        total_nitra,
        total_supply_count,
    })
}

fn generate_mission_damage(
    cached_mission_list: &[MissionCachedInfo],
    player_id_to_name: &HashMap<i16, String>,
    entity_game_id_to_name: HashMap<String, String>,
    mission_id: i32,
) -> Option<MissionDamageInfo> {
    let target_mission = cached_mission_list
        .iter()
        .find(|mission| mission.mission_info.id == mission_id)?;

    // causer -> taker -> amount
    let mut ff_causer_taker_map: HashMap<&String, HashMap<&String, f64>> =
        HashMap::with_capacity(target_mission.player_info.len());
    let mut ff_taker_causer_map: HashMap<&String, HashMap<&String, f64>> =
        HashMap::with_capacity(target_mission.player_info.len());

    let mut info: HashMap<String, PlayerDamageInfo> =
        HashMap::with_capacity(target_mission.player_info.len());

    for (causer_player_id, player_damage_map) in &target_mission.damage_info {
        let causer_player_name = player_id_to_name.get(causer_player_id).unwrap();

        for (taker_game_id, pack) in player_damage_map {
            if pack.taker_type != 1 {
                continue;
            }

            if pack.taker_id == *causer_player_id {
                continue;
            }

            ff_causer_taker_map
                .entry(causer_player_name)
                .or_insert_with(HashMap::new)
                .insert(taker_game_id, pack.total_amount);

            ff_taker_causer_map
                .entry(taker_game_id)
                .or_insert_with(HashMap::new)
                .insert(causer_player_name, pack.total_amount);
        }
    }

    for player_info in &target_mission.player_info {
        let player_id = player_info.player_id;
        let player_name = player_id_to_name.get(&player_id).unwrap();

        let player_damage = target_mission
            .damage_info
            .get(&player_id)
            .iter()
            .map(|x| x.iter())
            .flatten()
            .filter(|(_, pack)| pack.taker_type != 1)
            .map(|(k, v)| (k.clone(), v.total_amount))
            .collect::<HashMap<_, _>>();

        let player_kill = target_mission
            .kill_info
            .get(&player_id)
            .iter()
            .map(|x| x.iter())
            .flatten()
            .map(|(k, v)| (k.clone(), v.total_amount))
            .collect::<HashMap<_, _>>();

        let ff_data = PlayerFriendlyFireInfo {
            cause: ff_causer_taker_map
                .get(player_name)
                .map(|ff_map| {
                    ff_map
                        .into_iter()
                        .map(|(k, v)| ((*k).clone(), *v))
                        .collect()
                })
                .unwrap_or_else(HashMap::new),
            take: ff_taker_causer_map
                .get(player_name)
                .map(|ff_map| {
                    ff_map
                        .into_iter()
                        .map(|(k, v)| ((*k).clone(), *v))
                        .collect()
                })
                .unwrap_or_else(HashMap::new),
        };

        let supply_count = target_mission
            .supply_info
            .get(&player_id)
            .map(|player_supply_list| player_supply_list.len() as i16)
            .unwrap_or(0);

        info.insert(
            player_name.clone(),
            PlayerDamageInfo {
                damage: player_damage,
                kill: player_kill,
                ff: ff_data,
                supply_count,
            },
        );
    }

    Some(MissionDamageInfo {
        info,
        entity_mapping: entity_game_id_to_name,
    })
}

fn generate_mission_weapon_damage(
    cached_mission_list: &[MissionCachedInfo],
    weapon_game_id_to_character_game_id: &HashMap<String, String>,
    weapon_game_id_to_name: &HashMap<String, String>,
    mission_id: i32,
) -> Option<HashMap<String, MissionWeaponDamageInfo>> {
    let target_mission = cached_mission_list
        .iter()
        .find(|mission| mission.mission_info.id == mission_id)?;

    let mut result = HashMap::new();

    for (weapon_game_id, weapon_pack) in &target_mission.weapon_damage_info {
        let damage = weapon_pack
            .detail
            .values()
            .filter(|pack| pack.taker_type != 1)
            .map(|pack| pack.total_amount)
            .sum::<f64>();

        let friendly_fire = weapon_pack
            .detail
            .values()
            .filter(|pack| pack.taker_type == 1)
            .map(|pack| pack.total_amount)
            .sum::<f64>();

        let character_game_id = weapon_game_id_to_character_game_id
            .get(weapon_game_id)
            .map(|inner| inner.clone())
            .unwrap_or("Unknown".into());

        let mapped_name = weapon_game_id_to_name
            .get(weapon_game_id)
            .unwrap_or(weapon_game_id)
            .clone();

        result.insert(
            weapon_game_id.clone(),
            MissionWeaponDamageInfo {
                damage,
                friendly_fire,
                character_game_id,
                mapped_name,
            },
        );
    }

    Some(result)
}

fn generate_mission_resource(
    cached_mission_list: &[MissionCachedInfo],
    player_id_to_name: &HashMap<i16, String>,
    resource_game_id_to_name: &HashMap<String, String>,
    mission_id: i32,
) -> Option<MissionResourceInfo> {
    let target_mission = cached_mission_list
        .iter()
        .find(|mission| mission.mission_info.id == mission_id)?;
    let mut resource_info_by_player = HashMap::with_capacity(target_mission.player_info.len());

    for player_info in &target_mission.player_info {
        let player_id = player_info.player_id;
        let player_name = player_id_to_name.get(&player_id).unwrap();

        let resource_data = target_mission
            .resource_info
            .get(&player_id)
            .map(|player_resource_data| player_resource_data.clone())
            .unwrap_or_else(HashMap::new);

        let supply_data = target_mission
            .supply_info
            .get(&player_id)
            .map(|supply_list| supply_list.clone())
            .unwrap_or_else(Vec::new);

        resource_info_by_player.insert(
            player_name.clone(),
            PlayerResourceData {
                resource: resource_data,
                supply: supply_data,
            },
        );
    }

    Some(MissionResourceInfo {
        data: resource_info_by_player,
        resource_mapping: resource_game_id_to_name.clone(),
    })
}

pub fn generate_mission_kpi(
    mission_kpi_cached_info: &MissionKPICachedInfo,
    player_id_to_name: &HashMap<i16, String>,
    global_kpi_state: &CachedGlobalKPIState,
    kpi_config: &KPIConfig,
) -> Vec<MissionKPIInfo> {
    let mut result = Vec::with_capacity(mission_kpi_cached_info.raw_kpi_data.len());

    let mut mission_correction_factor_sum = HashMap::new();
    let mut mission_correction_factor = HashMap::new();

    for &kpi_component in CORRECTION_ITEMS {
        for character_type in mission_kpi_cached_info.player_id_to_kpi_character.values() {
            let correction_factor = global_kpi_state
                .character_correction_factor
                .get(character_type)
                .unwrap()
                .get(&kpi_component)
                .map(|x| x.correction_factor)
                .unwrap();

            *mission_correction_factor_sum
                .entry(kpi_component)
                .or_insert(0.0) += correction_factor;
        }
    }

    for &kpi_component in CORRECTION_ITEMS {
        mission_correction_factor.insert(
            kpi_component,
            mission_correction_factor_sum[&kpi_component]
                / global_kpi_state.standard_correction_sum[&kpi_component],
        );
    }

    for (player_id, raw_kpi_data) in &mission_kpi_cached_info.raw_kpi_data {
        let player_name = player_id_to_name.get(&player_id).unwrap().clone();

        let kpi_character_type = mission_kpi_cached_info
            .player_id_to_kpi_character
            .get(&player_id)
            .unwrap();

        let weighted_kill = raw_kpi_data
            .get(&KPIComponent::Kill)
            .unwrap()
            .weighted_value;
        let weighted_damage = raw_kpi_data
            .get(&KPIComponent::Damage)
            .unwrap()
            .weighted_value;
        let priority_damage = raw_kpi_data
            .get(&KPIComponent::Priority)
            .unwrap()
            .weighted_value;
        let revive_num = raw_kpi_data
            .get(&KPIComponent::Revive)
            .unwrap()
            .weighted_value;
        let death_num = raw_kpi_data
            .get(&KPIComponent::Death)
            .unwrap()
            .weighted_value;
        let friendly_fire = raw_kpi_data
            .get(&KPIComponent::FriendlyFire)
            .unwrap()
            .source_value;
        let nitra = raw_kpi_data
            .get(&KPIComponent::Nitra)
            .unwrap()
            .weighted_value;
        let supply_count = raw_kpi_data
            .get(&KPIComponent::Supply)
            .unwrap()
            .weighted_value;
        let weighted_resource = raw_kpi_data
            .get(&KPIComponent::Minerals)
            .unwrap()
            .weighted_value;

        let mut player_kpi_component_list = Vec::new();

        let mut player_mission_kpi_weighted_sum = 0.0;
        let mut player_mission_kpi_max_sum = 0.0;

        let mut component_name_to_component = HashMap::new();

        for (kpi_component, kpi_data) in raw_kpi_data {
            let component_name = kpi_component.to_string_zh();

            component_name_to_component.insert(component_name.clone(), kpi_component);

            let corrected_index = match mission_correction_factor.get(&kpi_component) {
                Some(factor) => (kpi_data.raw_index * factor).min(1.0),
                None => kpi_data.raw_index,
            };

            let transformed_index = match global_kpi_state
                .transform_range
                .get(kpi_character_type)
                .unwrap()
                .get(&kpi_component)
            {
                Some(range_info) => {
                    let mut range_index = 0;

                    for i in 0..range_info.len() {
                        if corrected_index > range_info[i].source_range.0 {
                            range_index = i;
                        } else {
                            break;
                        }
                    }

                    let transform_range = range_info[range_index];

                    corrected_index * transform_range.transform_cofficient.0
                        + transform_range.transform_cofficient.1
                }
                None => corrected_index,
            };

            let current_weight =
                kpi_config.character_component_weight[kpi_character_type][&kpi_component];

            player_kpi_component_list.push(MissionKPIComponent {
                name: component_name,
                source_value: kpi_data.source_value,
                weighted_value: kpi_data.weighted_value,
                mission_total_weighted_value: kpi_data.mission_total_weighted_value,
                raw_index: kpi_data.raw_index,
                corrected_index,
                transformed_index,
                weight: current_weight,
            });

            player_mission_kpi_weighted_sum += transformed_index * current_weight;
            player_mission_kpi_max_sum += kpi_component.max_value() * current_weight;
        }

        player_kpi_component_list.sort_unstable_by(|a, b| {
            let a_index: i16 = (**component_name_to_component.get(&a.name).unwrap()).into();
            let b_index: i16 = (**component_name_to_component.get(&b.name).unwrap()).into();

            a_index.cmp(&b_index)
        });

        result.push(MissionKPIInfo {
            player_name,
            kpi_character_type: kpi_character_type.to_string(),
            weighted_kill,
            weighted_damage,
            priority_damage,
            revive_num,
            death_num,
            friendly_fire,
            nitra,
            supply_count,
            weighted_resource,
            component: player_kpi_component_list,
            mission_kpi: player_mission_kpi_weighted_sum / player_mission_kpi_max_sum,
        });
    }

    result
}

#[get("/{mission_id}/info")]
async fn get_general_info(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    path: web::Path<i32>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<MissionGeneralInfo>> {
    let mission_id = path.into_inner();
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

        let invalid_mission_list = match mission_invalid::table
            .select(MissionInvalid::as_select())
            .load(&mut db_conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get invalid mission list: {}", e);
                return Err(());
            }
        };

        debug!("data prepared in {:?}", begin.elapsed());
        let begin = Instant::now();

        let result =
            generate_mission_general_info(&cached_mission_list, &invalid_mission_list, mission_id);

        debug!("mission general info generated in {:?}", begin.elapsed());

        Ok(result)
    })
    .await
    .unwrap();

    match result {
        Ok(x) => match x {
            Some(info) => Json(APIResponse::ok(info)),
            None => Json(APIResponse::not_found()),
        },
        Err(()) => Json(APIResponse::internal_error()),
    }
}

#[get("/{mission_id}/basic")]
async fn get_player_character(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    path: web::Path<i32>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<HashMap<String, String>>> {
    let mission_id = path.into_inner();
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

        debug!("data prepared in {:?}", begin.elapsed());
        let begin = Instant::now();

        let result = generate_mission_player_character(
            &cached_mission_list,
            &player_id_to_name,
            &character_id_to_game_id,
            mission_id,
        );

        debug!(
            "mission player character info generated in {:?}",
            begin.elapsed()
        );

        Ok(result)
    })
    .await
    .unwrap();

    match result {
        Ok(x) => match x {
            Some(info) => Json(APIResponse::ok(info)),
            None => Json(APIResponse::not_found()),
        },
        Err(()) => Json(APIResponse::internal_error()),
    }
}

#[get("/{mission_id}/general")]
async fn get_mission_general(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    path: web::Path<i32>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<MissionGeneralData>> {
    let mission_id = path.into_inner();
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

        let mission_type_list = match mission_type::table
            .select(MissionType::as_select())
            .load(&mut db_conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get mission type list: {}", e);
                return Err(());
            }
        };

        let mission_type_id_to_game_id = mission_type_list
            .into_iter()
            .map(|mission_type| (mission_type.id, mission_type.mission_type_game_id))
            .collect::<HashMap<_, _>>();

        debug!("data prepared in {:?}", begin.elapsed());
        let begin = Instant::now();

        let result = generate_mission_general(
            &cached_mission_list,
            &player_id_to_name,
            &character_id_to_game_id,
            &mission_type_id_to_game_id,
            mission_id,
        );

        debug!("mission general generated in {:?}", begin.elapsed());

        Ok(result)
    })
    .await
    .unwrap();

    match result {
        Ok(x) => match x {
            Some(info) => Json(APIResponse::ok(info)),
            None => Json(APIResponse::not_found()),
        },
        Err(()) => Json(APIResponse::internal_error()),
    }
}

#[get("/{mission_id}/damage")]
async fn get_mission_damage(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    path: web::Path<i32>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<MissionDamageInfo>> {
    let mission_id = path.into_inner();
    let mapping = app_state.mapping.lock().unwrap();

    let entity_blacklist_set = mapping.entity_blacklist_set.clone();
    let entity_combine = mapping.entity_combine.clone();
    let weapon_combine = mapping.weapon_combine.clone();
    let entity_game_id_to_name = mapping.entity_mapping.clone();

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

        debug!("data prepared in {:?}", begin.elapsed());
        let begin = Instant::now();

        let result = generate_mission_damage(
            &cached_mission_list,
            &player_id_to_name,
            entity_game_id_to_name,
            mission_id,
        );

        debug!("mission damage generated in {:?}", begin.elapsed());

        Ok(result)
    })
    .await
    .unwrap();

    match result {
        Ok(x) => match x {
            Some(info) => Json(APIResponse::ok(info)),
            None => Json(APIResponse::not_found()),
        },
        Err(()) => Json(APIResponse::internal_error()),
    }
}

#[get("/{mission_id}/weapon")]
async fn get_mission_weapon_damage(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    path: web::Path<i32>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<HashMap<String, MissionWeaponDamageInfo>>> {
    let mission_id = path.into_inner();
    let mapping = app_state.mapping.lock().unwrap();

    let entity_blacklist_set = mapping.entity_blacklist_set.clone();
    let entity_combine = mapping.entity_combine.clone();
    let weapon_combine = mapping.weapon_combine.clone();
    let weapon_game_id_to_name = mapping.weapon_mapping.clone();
    let weapon_game_id_to_character_game_id = mapping.weapon_character.clone();

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

        debug!("data prepared in {:?}", begin.elapsed());
        let begin = Instant::now();

        let result = generate_mission_weapon_damage(
            &cached_mission_list,
            &weapon_game_id_to_character_game_id,
            &weapon_game_id_to_name,
            mission_id,
        );

        debug!("mission weapon damage generated in {:?}", begin.elapsed());

        Ok(result)
    })
    .await
    .unwrap();

    match result {
        Ok(x) => match x {
            Some(info) => Json(APIResponse::ok(info)),
            None => Json(APIResponse::not_found()),
        },
        Err(()) => Json(APIResponse::internal_error()),
    }
}

#[get("/{mission_id}/resource")]
async fn get_mission_resource_info(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    path: web::Path<i32>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<MissionResourceInfo>> {
    let mission_id = path.into_inner();
    let mapping = app_state.mapping.lock().unwrap();

    let entity_blacklist_set = mapping.entity_blacklist_set.clone();
    let entity_combine = mapping.entity_combine.clone();
    let weapon_combine = mapping.weapon_combine.clone();
    let resource_game_id_to_name = mapping.resource_mapping.clone();

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

        debug!("data prepared in {:?}", begin.elapsed());
        let begin = Instant::now();

        let result = generate_mission_resource(
            &cached_mission_list,
            &player_id_to_name,
            &resource_game_id_to_name,
            mission_id,
        );

        debug!("mission resource info generated in {:?}", begin.elapsed());

        Ok(result)
    })
    .await
    .unwrap();

    match result {
        Ok(x) => match x {
            Some(info) => Json(APIResponse::ok(info)),
            None => Json(APIResponse::not_found()),
        },
        Err(()) => Json(APIResponse::internal_error()),
    }
}

#[get("/{mission_id}/kpi")]
async fn get_mission_kpi(
    app_state: Data<AppState>,
    db_pool: Data<DbPool>,
    path: web::Path<i32>,
    redis_client: Data<redis::Client>,
) -> Json<APIResponse<Vec<MissionKPIInfo>>> {
    let mission_id = path.into_inner();
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

        let mut found = false;

        for mission in &cached_mission_list {
            if mission.mission_info.id == mission_id {
                found = true;
                break;
            }
        }

        if !found {
            return Ok(None);
        }

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

        let invalid_mission_id_list: Vec<i32> = match mission_invalid::table
            .select(mission_invalid::mission_id)
            .load(&mut db_conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get invalid mission list: {}", e);
                return Err(());
            }
        };

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

        let global_kpi_state = match CachedGlobalKPIState::get_cached(
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
        ) {
            Ok(x) => x,
            Err(()) => {
                error!("cannot get global kpi state");
                return Err(());
            }
        };

        let mission_kpi_cached_info = match MissionKPICachedInfo::get_cached(
            &mut db_conn,
            &mut redis_conn,
            &entity_blacklist_set,
            &entity_combine,
            &weapon_combine,
            &character_id_to_game_id,
            &player_id_to_name,
            &scout_special_player_set,
            &kpi_config,
            mission_id,
        ) {
            Ok(x) => x,
            Err(()) => {
                error!("cannot get mission kpi cached info");
                return Err(());
            }
        };

        debug!("data prepared in {:?}", begin.elapsed());
        let begin = Instant::now();

        let result = generate_mission_kpi(
            &mission_kpi_cached_info,
            &player_id_to_name,
            &global_kpi_state,
            &kpi_config,
        );

        debug!("mission kpi generated in {:?}", begin.elapsed());

        Ok(Some(result))
    })
    .await
    .unwrap();

    match result {
        Ok(x) => match x {
            Some(info) => Json(APIResponse::ok(info)),
            None => Json(APIResponse::not_found()),
        },
        Err(()) => Json(APIResponse::internal_error()),
    }
}
