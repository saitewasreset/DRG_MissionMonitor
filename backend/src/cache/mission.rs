use crate::damage::{DamagePack, KillPack, SupplyPack, WeaponPack};
use crate::db::models::*;
use crate::db::schema::*;
use crate::kpi::{
    apply_weight_table, friendly_fire_index, CharacterKPIType, KPIComponent, KPIConfig,
};
use crate::{FLOAT_EPSILON, NITRA_GAME_ID};
use diesel::prelude::*;
use diesel::{PgConnection, RunQueryDsl};
use log::{debug, error, info, warn};
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

// 用于缓存输出任务详情及计算任务KPI、玩家KPI、赋分信息等需要的任务信息
// depends on:
// - mapping: entity_blacklist, entity_combine, weapon_combine

#[derive(Serialize, Deserialize)]
pub struct MissionCachedInfo {
    pub mission_info: Mission,
    pub player_info: Vec<PlayerInfo>,
    // player_id -> index
    pub player_index: HashMap<i16, f64>,
    // player_id -> info
    pub kill_info: HashMap<i16, HashMap<String, KillPack>>,
    // player_id -> info
    pub damage_info: HashMap<i16, HashMap<String, DamagePack>>,
    pub weapon_damage_info: HashMap<String, WeaponPack>,
    // player_id -> resource_game_id -> total_amount
    pub resource_info: HashMap<i16, HashMap<String, f64>>,
    // player_id -> count
    pub revive_count: HashMap<i16, i16>,
    // player_id -> count
    pub death_count: HashMap<i16, i16>,
    // player_id -> info
    pub supply_info: HashMap<i16, Vec<SupplyPack>>,
}

impl MissionCachedInfo {
    fn generate(
        mission_info: &Mission,
        player_info_list: &Vec<PlayerInfo>,
        raw_kill_info_list: &Vec<KillInfo>,
        raw_damage_info_list: &Vec<DamageInfo>,
        raw_resource_info_list: &Vec<ResourceInfo>,
        raw_supply_info_list: &Vec<SupplyInfo>,
        entity_blacklist_set: &HashSet<String>,
        entity_combine: &HashMap<String, String>,
        weapon_combine: &HashMap<String, String>,
        id_to_player_name: &HashMap<i16, String>,
        id_to_entity_game_id: &HashMap<i16, String>,
        id_to_weapon_game_id: &HashMap<i16, String>,
        id_to_resource_game_id: &HashMap<i16, String>,
    ) -> (Self, Duration) {
        let begin = Instant::now();

        let mut player_index = HashMap::with_capacity(player_info_list.len());
        let mut revive_count = HashMap::with_capacity(player_info_list.len());
        let mut death_count = HashMap::with_capacity(player_info_list.len());

        for current_player_info in player_info_list {
            player_index.insert(
                current_player_info.player_id,
                current_player_info.present_time as f64 / mission_info.mission_time as f64,
            );
            revive_count.insert(
                current_player_info.player_id,
                current_player_info.revive_num,
            );
            death_count.insert(current_player_info.player_id, current_player_info.death_num);
        }

        let mut kill_info = HashMap::with_capacity(player_info_list.len());

        for current_kill_info in raw_kill_info_list {
            let record_entity_game_id = id_to_entity_game_id
                .get(&current_kill_info.entity_id)
                .unwrap();

            let killed_entity_game_id = entity_combine
                .get(record_entity_game_id)
                .unwrap_or(record_entity_game_id);

            if entity_blacklist_set.contains(killed_entity_game_id) {
                continue;
            }

            let player_kill_map = kill_info
                .entry(current_kill_info.player_id)
                .or_insert(HashMap::new());

            let entity_kill_entry =
                player_kill_map
                    .entry(killed_entity_game_id)
                    .or_insert(KillPack {
                        taker_id: current_kill_info.entity_id,
                        taker_name: killed_entity_game_id.clone(),
                        total_amount: 0,
                    });

            entity_kill_entry.total_amount += 1;
        }

        let weapon_game_id_to_id = id_to_weapon_game_id
            .iter()
            .map(|(&k, v)| (v, k))
            .collect::<HashMap<_, _>>();

        let mut damage_info = HashMap::with_capacity(player_info_list.len());

        let mut weapon_details = HashMap::new();

        for current_damage_info in raw_damage_info_list {
            // 0→unknown 1→ player 2→enemy
            if current_damage_info.causer_type != 1 {
                continue;
            }

            let (taker_game_id, taker_type) = match current_damage_info.taker_type {
                1 => (
                    id_to_player_name
                        .get(&current_damage_info.taker_id)
                        .unwrap(),
                    1,
                ),
                x => {
                    let record_entity_game_id = id_to_entity_game_id
                        .get(&current_damage_info.taker_id)
                        .unwrap();

                    let entity_game_id = entity_combine
                        .get(record_entity_game_id)
                        .unwrap_or(record_entity_game_id);

                    if entity_blacklist_set.contains(entity_game_id) {
                        continue;
                    }

                    (entity_game_id, x)
                }
            };

            let player_damage_map = damage_info
                .entry(current_damage_info.causer_id)
                .or_insert(HashMap::new());

            let player_damage_entry =
                player_damage_map
                    .entry(taker_game_id)
                    .or_insert(DamagePack {
                        taker_id: current_damage_info.taker_id,
                        taker_type,
                        weapon_id: current_damage_info.weapon_id,
                        total_amount: 0.0,
                    });
            player_damage_entry.total_amount += current_damage_info.damage;

            let record_weapon_game_id = id_to_weapon_game_id
                .get(&current_damage_info.weapon_id)
                .unwrap();

            let weapon_game_id = weapon_combine
                .get(record_weapon_game_id)
                .unwrap_or(record_weapon_game_id);

            let detail_map = weapon_details
                .entry(weapon_game_id)
                .or_insert(HashMap::new());

            let detail_entry = detail_map.entry(taker_game_id).or_insert(DamagePack {
                taker_id: current_damage_info.taker_id,
                taker_type,
                weapon_id: current_damage_info.weapon_id,
                total_amount: 0.0,
            });

            detail_entry.total_amount += current_damage_info.damage;
        }

        let mut resource_info = HashMap::with_capacity(player_info_list.len());

        for current_resource_info in raw_resource_info_list {
            let resource_game_id = id_to_resource_game_id
                .get(&current_resource_info.resource_id)
                .unwrap();

            let player_resource_info_map = resource_info
                .entry(current_resource_info.player_id)
                .or_insert(HashMap::new());

            let resource_entry = player_resource_info_map
                .entry(resource_game_id)
                .or_insert(0.0);

            *resource_entry += current_resource_info.amount;
        }

        let mut supply_info = HashMap::with_capacity(player_info_list.len());

        for current_supply_info in raw_supply_info_list {
            let player_supply_list = supply_info
                .entry(current_supply_info.player_id)
                .or_insert(Vec::new());

            player_supply_list.push(SupplyPack {
                ammo: current_supply_info.ammo,
                health: current_supply_info.health,
            })
        }

        let weapon_damage_info = weapon_details
            .into_iter()
            .map(|(weapon_game_id, detail)| {
                let weapon_id = weapon_game_id_to_id.get(weapon_game_id).unwrap();
                let total_damage = detail
                    .values()
                    .into_iter()
                    .map(|v| v.total_amount)
                    .sum::<f64>();
                let detail_map = detail
                    .into_iter()
                    .map(|(k, v)| (k.clone(), v))
                    .collect::<HashMap<_, _>>();

                (
                    weapon_game_id.clone(),
                    WeaponPack {
                        weapon_id: *weapon_id,
                        total_amount: total_damage,
                        detail: detail_map,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        // Convert inner HashMap<&String, _> to HashMap<String, _>
        let kill_info = kill_info
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    v.into_iter()
                        .map(|(inner_k, inner_v)| (inner_k.clone(), inner_v))
                        .collect::<HashMap<_, _>>(),
                )
            })
            .collect::<HashMap<_, _>>();

        let damage_info = damage_info
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    v.into_iter()
                        .map(|(inner_k, inner_v)| (inner_k.clone(), inner_v))
                        .collect::<HashMap<_, _>>(),
                )
            })
            .collect::<HashMap<_, _>>();

        let weapon_damage_info = weapon_damage_info
            .into_iter()
            .map(|(k, v)| (k.clone(), v))
            .collect::<HashMap<_, _>>();

        let resource_info = resource_info
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    v.into_iter()
                        .map(|(inner_k, inner_v)| (inner_k.clone(), inner_v))
                        .collect::<HashMap<_, _>>(),
                )
            })
            .collect::<HashMap<_, _>>();

        let elapsed = begin.elapsed();

        debug!(
            "generated cached mission info for {} in {:?}",
            mission_info.id, elapsed
        );

        (
            MissionCachedInfo {
                mission_info: mission_info.clone(),
                player_info: player_info_list.clone(),
                player_index,
                kill_info,
                damage_info,
                weapon_damage_info,
                resource_info,
                revive_count,
                death_count,
                supply_info,
            },
            elapsed,
        )
    }

    pub fn from_db(
        conn: &mut PgConnection,
        entity_blacklist_set: &HashSet<String>,
        entity_combine: &HashMap<String, String>,
        weapon_combine: &HashMap<String, String>,
        mission_id: i32,
    ) -> Result<Self, ()> {
        let begin = Instant::now();

        let player_list: Vec<Player> = match player::table.load(conn) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot load player from db: {}", e);
                return Err(());
            }
        };

        let entity_list: Vec<Entity> = match entity::table.load(conn) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot load entity from db: {}", e);
                return Err(());
            }
        };

        let resource_list: Vec<Resource> = match resource::table.load(conn) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot load resource from db: {}", e);
                return Err(());
            }
        };

        let weapon_list: Vec<Weapon> = match weapon::table.load(conn) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot load weapon from db: {}", e);
                return Err(());
            }
        };

        let id_to_player_name = player_list
            .into_iter()
            .map(|player| (player.id, player.player_name))
            .collect::<HashMap<_, _>>();

        let id_to_entity_game_id = entity_list
            .into_iter()
            .map(|entity| (entity.id, entity.entity_game_id))
            .collect::<HashMap<_, _>>();

        let id_to_resource_game_id = resource_list
            .into_iter()
            .map(|resource| (resource.id, resource.resource_game_id))
            .collect::<HashMap<_, _>>();

        let id_to_weapon_game_id = weapon_list
            .into_iter()
            .map(|weapon| (weapon.id, weapon.weapon_game_id))
            .collect::<HashMap<_, _>>();

        let mission_info: Mission = match mission::table
            .filter(mission::id.eq(mission_id))
            .get_result(conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!("cannot load mission_id = {} from db: {}", mission_id, e);
                return Err(());
            }
        };

        let player_info: Vec<PlayerInfo> = match PlayerInfo::belonging_to(&mission_info).load(conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!(
                    "cannot load player info for mission_id = {} from db: {}",
                    mission_id, e
                );
                return Err(());
            }
        };

        let damage_info: Vec<DamageInfo> = match DamageInfo::belonging_to(&mission_info).load(conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!(
                    "cannot load damage info for mission_id = {} from db: {}",
                    mission_id, e
                );
                return Err(());
            }
        };

        let kill_info: Vec<KillInfo> = match KillInfo::belonging_to(&mission_info).load(conn) {
            Ok(x) => x,
            Err(e) => {
                error!(
                    "cannot load kill info for mission_id = {} from db: {}",
                    mission_id, e
                );
                return Err(());
            }
        };

        let resource_info: Vec<ResourceInfo> =
            match ResourceInfo::belonging_to(&mission_info).load(conn) {
                Ok(x) => x,
                Err(e) => {
                    error!(
                        "cannot load resource info for mission_id = {} from db: {}",
                        mission_id, e
                    );
                    return Err(());
                }
            };

        let supply_info: Vec<SupplyInfo> = match SupplyInfo::belonging_to(&mission_info).load(conn)
        {
            Ok(x) => x,
            Err(e) => {
                error!(
                    "cannot load supply info for mission_id = {} from db: {}",
                    mission_id, e
                );
                return Err(());
            }
        };

        let load_from_db_elapsed = begin.elapsed();

        let (result, generate_elapsed) = Self::generate(
            &mission_info,
            &player_info,
            &kill_info,
            &damage_info,
            &resource_info,
            &supply_info,
            entity_blacklist_set,
            entity_combine,
            weapon_combine,
            &id_to_player_name,
            &id_to_entity_game_id,
            &id_to_weapon_game_id,
            &id_to_resource_game_id,
        );

        info!("generated cached mission info from db for {} in {:?}(total) = {:?}(load_from_db) + {:?}(generate)", mission_id, load_from_db_elapsed + generate_elapsed, load_from_db_elapsed, generate_elapsed);

        Ok(result)
    }

    pub fn from_db_all(
        conn: &mut PgConnection,
        entity_blacklist_set: &HashSet<String>,
        entity_combine: &HashMap<String, String>,
        weapon_combine: &HashMap<String, String>,
    ) -> Result<Vec<Self>, ()> {
        let begin = Instant::now();

        let player_list: Vec<Player> = match player::table.load(conn) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot load player from db: {}", e);
                return Err(());
            }
        };

        let entity_list: Vec<Entity> = match entity::table.load(conn) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot load entity from db: {}", e);
                return Err(());
            }
        };

        let resource_list: Vec<Resource> = match resource::table.load(conn) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot load resource from db: {}", e);
                return Err(());
            }
        };

        let weapon_list: Vec<Weapon> = match weapon::table.load(conn) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot load weapon from db: {}", e);
                return Err(());
            }
        };

        let id_to_player_name = player_list
            .into_iter()
            .map(|player| (player.id, player.player_name))
            .collect::<HashMap<_, _>>();

        let id_to_entity_game_id = entity_list
            .into_iter()
            .map(|entity| (entity.id, entity.entity_game_id))
            .collect::<HashMap<_, _>>();

        let id_to_resource_game_id = resource_list
            .into_iter()
            .map(|resource| (resource.id, resource.resource_game_id))
            .collect::<HashMap<_, _>>();

        let id_to_weapon_game_id = weapon_list
            .into_iter()
            .map(|weapon| (weapon.id, weapon.weapon_game_id))
            .collect::<HashMap<_, _>>();

        let all_mission_info = match mission::table.select(Mission::as_select()).load(conn) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot load missions from db: {}", e);
                return Err(());
            }
        };

        let all_player_info: Vec<PlayerInfo> =
            match PlayerInfo::belonging_to(&all_mission_info).load(conn) {
                Ok(x) => x,
                Err(e) => {
                    error!("cannot load player info from db: {}", e);
                    return Err(());
                }
            };

        let all_damage_info: Vec<DamageInfo> =
            match DamageInfo::belonging_to(&all_mission_info).load(conn) {
                Ok(x) => x,
                Err(e) => {
                    error!("cannot load damage info from db: {}", e);
                    return Err(());
                }
            };

        let all_kill_info: Vec<KillInfo> =
            match KillInfo::belonging_to(&all_mission_info).load(conn) {
                Ok(x) => x,
                Err(e) => {
                    error!("cannot load kill info from db: {}", e);
                    return Err(());
                }
            };

        let all_resource_info: Vec<ResourceInfo> =
            match ResourceInfo::belonging_to(&all_mission_info).load(conn) {
                Ok(x) => x,
                Err(e) => {
                    error!("cannot load resource info from db: {}", e);
                    return Err(());
                }
            };

        let all_supply_info: Vec<SupplyInfo> =
            match SupplyInfo::belonging_to(&all_mission_info).load(conn) {
                Ok(x) => x,
                Err(e) => {
                    error!("cannot load supply info from db: {}", e);
                    return Err(());
                }
            };

        let load_from_db_elapsed = begin.elapsed();
        let begin = Instant::now();

        let player_info_by_mission = all_player_info
            .grouped_by(&all_mission_info)
            .into_iter()
            .zip(&all_mission_info)
            .map(|(children, parent)| (parent.id, children))
            .collect::<HashMap<_, _>>();

        let damage_info_by_mission = all_damage_info
            .grouped_by(&all_mission_info)
            .into_iter()
            .zip(&all_mission_info)
            .map(|(children, parent)| (parent.id, children))
            .collect::<HashMap<_, _>>();

        let kill_info_by_mission = all_kill_info
            .grouped_by(&all_mission_info)
            .into_iter()
            .zip(&all_mission_info)
            .map(|(children, parent)| (parent.id, children))
            .collect::<HashMap<_, _>>();

        let resource_info_by_mission = all_resource_info
            .grouped_by(&all_mission_info)
            .into_iter()
            .zip(&all_mission_info)
            .map(|(children, parent)| (parent.id, children))
            .collect::<HashMap<_, _>>();

        let supply_info_by_mission = all_supply_info
            .grouped_by(&all_mission_info)
            .into_iter()
            .zip(&all_mission_info)
            .map(|(children, parent)| (parent.id, children))
            .collect::<HashMap<_, _>>();

        let result = all_mission_info
            .iter()
            .map(|mission| {
                Self::generate(
                    mission,
                    player_info_by_mission.get(&mission.id).unwrap(),
                    kill_info_by_mission.get(&mission.id).unwrap(),
                    damage_info_by_mission.get(&mission.id).unwrap(),
                    resource_info_by_mission.get(&mission.id).unwrap(),
                    supply_info_by_mission.get(&mission.id).unwrap(),
                    &entity_blacklist_set,
                    &entity_combine,
                    &weapon_combine,
                    &id_to_player_name,
                    &id_to_entity_game_id,
                    &id_to_weapon_game_id,
                    &id_to_resource_game_id,
                )
                .0
            })
            .collect::<Vec<_>>();

        let generate_elapsed = begin.elapsed();

        info!("generated {} cached mission info from db in {:?}(total) = {:?}(load_from_db) + {:?}(generate)", result.len(), load_from_db_elapsed + generate_elapsed, load_from_db_elapsed, generate_elapsed);

        Ok(result)
    }

    pub fn get_cached(
        db_conn: &mut PgConnection,
        redis_conn: &mut redis::Connection,
        entity_blacklist_set: &HashSet<String>,
        entity_combine: &HashMap<String, String>,
        weapon_combine: &HashMap<String, String>,
        mission_id: i32,
    ) -> Result<Self, ()> {
        let cached_bytes: Option<Vec<u8>> =
            redis_conn.get(format!("mission_raw:{}", mission_id)).ok();

        let cached_content = match cached_bytes {
            Some(x) => {
                let decoded: MissionCachedInfo = match rmp_serde::from_read(&x[..]) {
                    Ok(x) => x,
                    Err(e) => {
                        error!("cannot decode cached bytes: {}", e);
                        return Err(());
                    }
                };

                decoded
            }
            None => {
                match Self::from_db(
                    db_conn,
                    entity_blacklist_set,
                    entity_combine,
                    weapon_combine,
                    mission_id,
                ) {
                    Ok(x) => {
                        let serialized = rmp_serde::to_vec(&x).unwrap();
                        match redis_conn.set(format!("mission_raw:{}", mission_id), serialized) {
                            Ok(()) => x,
                            Err(e) => {
                                error!("cannot write data to redis: {}", e);
                                return Err(());
                            }
                        }
                    }
                    Err(()) => return Err(()),
                }
            }
        };

        Ok(cached_content)
    }

    pub fn get_cached_all(
        db_conn: &mut PgConnection,
        redis_conn: &mut redis::Connection,
        entity_blacklist_set: &HashSet<String>,
        entity_combine: &HashMap<String, String>,
        weapon_combine: &HashMap<String, String>,
    ) -> Result<Vec<Self>, ()> {
        let mission_list = match mission::table.select(Mission::as_select()).load(db_conn) {
            Ok(x) => x,
            Err(e) => {
                error!("cannot get mission list from db: {}", e);
                return Err(());
            }
        };

        let mut result = Vec::with_capacity(mission_list.len());

        for mission in mission_list {
            let redis_key = format!("mission_raw:{}", mission.id);

            let cached_info = match redis_conn.get::<_, Vec<u8>>(&redis_key) {
                Ok(x) => match rmp_serde::from_slice(&x[..]) {
                    Ok(x) => x,
                    Err(e) => {
                        error!("cannot decode cached bytes: {}", e);
                        return Err(());
                    }
                },
                Err(e) => {
                    warn!("cannot get mission {} from redis: {}", mission.id, e);

                    match Self::from_db(
                        db_conn,
                        entity_blacklist_set,
                        entity_combine,
                        weapon_combine,
                        mission.id,
                    ) {
                        Ok(x) => {
                            let serialized = rmp_serde::to_vec(&x).unwrap();
                            if redis_conn
                                .set::<_, Vec<u8>, ()>(&redis_key, serialized)
                                .is_err()
                            {
                                error!("cannot write data to redis: {}", e);
                                return Err(());
                            }
                            x
                        }
                        Err(()) => {
                            return Err(());
                        }
                    }
                }
            };

            result.push(cached_info);
        }

        Ok(result)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct PlayerRawKPIData {
    pub source_value: f64,
    pub weighted_value: f64,
    pub mission_total_weighted_value: f64,
    pub raw_index: f64,
}

#[derive(Serialize, Deserialize)]

// depends on:
// - MissionCachedInfo
// - KPIConfig
// - mapping: scout_special_player
pub struct MissionKPICachedInfo {
    pub mission_id: i32,
    pub damage_map: HashMap<i16, HashMap<String, f64>>,
    pub kill_map: HashMap<i16, HashMap<String, f64>>,
    pub resource_map: HashMap<i16, HashMap<String, f64>>,
    pub total_damage_map: HashMap<String, f64>,
    pub total_kill_map: HashMap<String, f64>,
    pub total_resource_map: HashMap<String, f64>,
    pub player_id_to_kpi_character: HashMap<i16, CharacterKPIType>,
    pub raw_kpi_data: HashMap<i16, HashMap<KPIComponent, PlayerRawKPIData>>,
}

impl MissionKPICachedInfo {
    fn generate(
        mission_info: &MissionCachedInfo,
        character_id_to_game_id: &HashMap<i16, String>,
        player_id_to_name: &HashMap<i16, String>,
        scout_special_player_set: &HashSet<String>,
        kpi_config: &KPIConfig,
    ) -> (Self, Duration) {
        let begin = Instant::now();

        let damage_map = mission_info
            .damage_info
            .iter()
            .map(|(player_id, player_data)| {
                (
                    *player_id,
                    player_data
                        .iter()
                        .filter(|(_, pack)| pack.taker_type != 1)
                        .map(|(k, v)| (k.clone(), v.total_amount))
                        .collect::<HashMap<_, _>>(),
                )
            })
            .collect::<HashMap<_, _>>();

        let kill_map = mission_info
            .kill_info
            .iter()
            .map(|(player_id, player_data)| {
                (
                    *player_id,
                    player_data
                        .iter()
                        .map(|(k, v)| (k.clone(), v.total_amount as f64))
                        .collect::<HashMap<_, _>>(),
                )
            })
            .collect::<HashMap<_, _>>();

        let resource_map = &mission_info.resource_info;

        let mut total_damage_map = HashMap::new();
        let mut total_kill_map = HashMap::new();
        let mut total_resource_map = HashMap::new();

        for (taker_id, value) in damage_map.values().map(|v| v.iter()).flatten() {
            *total_damage_map.entry(taker_id.clone()).or_insert(0.0) += *value;
        }

        for (killer_id, value) in kill_map.values().map(|v| v.iter()).flatten() {
            *total_kill_map.entry(killer_id.clone()).or_insert(0.0) += *value;
        }

        for (resource_id, value) in resource_map.values().map(|v| v.iter()).flatten() {
            *total_resource_map.entry(resource_id.clone()).or_insert(0.0) += *value;
        }

        let total_weighted_resource_map =
            apply_weight_table(&total_resource_map, &kpi_config.resource_weight_table);

        let mut player_id_to_kpi_character = HashMap::with_capacity(mission_info.player_info.len());

        let total_revive_count = mission_info
            .player_info
            .iter()
            .map(|player_info| player_info.revive_num)
            .sum::<i16>() as f64;

        let total_death_count = mission_info
            .player_info
            .iter()
            .map(|player_info| player_info.death_num)
            .sum::<i16>() as f64;

        let total_supply_count = mission_info
            .supply_info
            .iter()
            .map(|(_, supply_list)| supply_list.len())
            .sum::<usize>() as f64;

        let mut raw_kpi_data = HashMap::new();

        for player_info in &mission_info.player_info {
            let player_name = player_id_to_name.get(&player_info.player_id).unwrap();
            let player_character_game_id = character_id_to_game_id
                .get(&player_info.character_id)
                .unwrap();

            let player_character_kpi_type = CharacterKPIType::from_player(
                player_character_game_id,
                player_name,
                scout_special_player_set,
            );

            player_id_to_kpi_character.insert(player_info.player_id, player_character_kpi_type);

            let character_weight_table = kpi_config
                .character_weight_table
                .get(&player_character_kpi_type)
                .map_or(HashMap::new(), |x| x.clone());
            // Kill

            let source_kill = kill_map
                .get(&player_info.player_id)
                .unwrap_or(&HashMap::new())
                .values()
                .sum::<f64>();

            let weighted_kill_map = apply_weight_table(
                kill_map
                    .get(&player_info.player_id)
                    .unwrap_or(&HashMap::new()),
                &character_weight_table,
            );

            let weighted_kill = weighted_kill_map.values().sum::<f64>();
            let mission_total_weighted_kill =
                apply_weight_table(&total_kill_map, &character_weight_table)
                    .values()
                    .sum::<f64>();

            // Damage

            let source_damage = damage_map
                .get(&player_info.player_id)
                .unwrap_or(&HashMap::new())
                .values()
                .sum::<f64>();

            let weighted_damage_map = apply_weight_table(
                damage_map
                    .get(&player_info.player_id)
                    .unwrap_or(&HashMap::new()),
                &character_weight_table,
            );

            let weighted_damage = weighted_damage_map.values().sum::<f64>();
            let mission_total_weighted_damage =
                apply_weight_table(&total_damage_map, &character_weight_table)
                    .values()
                    .sum::<f64>();

            // Priority
            let priority_map = apply_weight_table(
                damage_map
                    .get(&player_info.player_id)
                    .unwrap_or(&HashMap::new()),
                &kpi_config.priority_table,
            );

            let priority_damage = priority_map.values().sum::<f64>();
            let mission_total_priority_damage =
                apply_weight_table(&total_damage_map, &kpi_config.priority_table)
                    .values()
                    .sum::<f64>();

            // Revive

            let player_revive_count = player_info.revive_num as f64;

            // Death

            let player_death_count = player_info.death_num as f64;

            // FriendlyFire

            let player_friendly_fire = mission_info
                .damage_info
                .get(&player_info.player_id)
                .unwrap_or(&HashMap::new())
                .iter()
                .filter(|(_, pack)| pack.taker_type == 1 && pack.taker_id != player_info.player_id)
                .map(|(_, pack)| pack.total_amount)
                .sum::<f64>();

            let player_overall_damage = source_damage + player_friendly_fire;

            let player_ff_index = match player_overall_damage {
                0.0..FLOAT_EPSILON => 1.0,
                _ => friendly_fire_index(player_friendly_fire / player_overall_damage),
            };

            // Nitra

            let player_nitra = *resource_map
                .get(&player_info.player_id)
                .unwrap_or(&HashMap::new())
                .get(NITRA_GAME_ID)
                .unwrap_or(&0.0);

            let total_nitra = *total_resource_map.get(NITRA_GAME_ID).unwrap_or(&0.0);

            // Minerals

            let player_source_minerals = resource_map
                .get(&player_info.player_id)
                .unwrap_or(&HashMap::new())
                .values()
                .sum::<f64>();

            let player_weighted_minerals = apply_weight_table(
                resource_map
                    .get(&player_info.player_id)
                    .unwrap_or(&HashMap::new()),
                &kpi_config.resource_weight_table,
            )
            .values()
            .sum::<f64>();

            let total_weighted_minerals = total_weighted_resource_map.values().sum::<f64>();

            // Supply

            let player_supply_count = mission_info
                .supply_info
                .get(&player_info.player_id)
                .unwrap_or(&Vec::new())
                .len() as f64;

            let mut player_raw_kpi_data = HashMap::new();

            player_raw_kpi_data.insert(
                KPIComponent::Kill,
                PlayerRawKPIData {
                    source_value: source_kill,
                    weighted_value: weighted_kill,
                    mission_total_weighted_value: mission_total_weighted_kill,
                    raw_index: match mission_total_weighted_kill {
                        0.0..FLOAT_EPSILON => 0.0,
                        _ => weighted_kill / mission_total_weighted_kill,
                    },
                },
            );

            player_raw_kpi_data.insert(
                KPIComponent::Damage,
                PlayerRawKPIData {
                    source_value: source_damage,
                    weighted_value: weighted_damage,
                    mission_total_weighted_value: mission_total_weighted_damage,
                    raw_index: match mission_total_weighted_damage {
                        0.0..FLOAT_EPSILON => 0.0,
                        _ => weighted_damage / mission_total_weighted_damage,
                    },
                },
            );

            player_raw_kpi_data.insert(
                KPIComponent::Priority,
                PlayerRawKPIData {
                    source_value: source_damage,
                    weighted_value: priority_damage,
                    mission_total_weighted_value: mission_total_priority_damage,
                    raw_index: match mission_total_priority_damage {
                        0.0..FLOAT_EPSILON => 0.0,
                        _ => priority_damage / mission_total_priority_damage,
                    },
                },
            );

            player_raw_kpi_data.insert(
                KPIComponent::Revive,
                PlayerRawKPIData {
                    source_value: player_revive_count,
                    weighted_value: player_revive_count,
                    mission_total_weighted_value: total_revive_count,
                    raw_index: match total_revive_count {
                        0.0..FLOAT_EPSILON => 1.0,
                        _ => player_revive_count / total_revive_count,
                    },
                },
            );

            player_raw_kpi_data.insert(
                KPIComponent::Death,
                PlayerRawKPIData {
                    source_value: player_death_count,
                    weighted_value: player_death_count,
                    mission_total_weighted_value: total_death_count,
                    raw_index: match total_death_count {
                        0.0..FLOAT_EPSILON => 0.0,
                        _ => -player_death_count / total_death_count,
                    },
                },
            );

            player_raw_kpi_data.insert(
                KPIComponent::FriendlyFire,
                PlayerRawKPIData {
                    source_value: player_friendly_fire,
                    weighted_value: player_ff_index,
                    mission_total_weighted_value: 0.0,
                    raw_index: player_ff_index,
                },
            );

            player_raw_kpi_data.insert(
                KPIComponent::Nitra,
                PlayerRawKPIData {
                    source_value: player_nitra,
                    weighted_value: player_nitra,
                    mission_total_weighted_value: total_nitra,
                    raw_index: match total_nitra {
                        0.0..FLOAT_EPSILON => 0.0,
                        _ => player_nitra / total_nitra,
                    },
                },
            );

            player_raw_kpi_data.insert(
                KPIComponent::Minerals,
                PlayerRawKPIData {
                    source_value: player_source_minerals,
                    weighted_value: player_weighted_minerals,
                    mission_total_weighted_value: total_weighted_minerals,
                    raw_index: match total_weighted_minerals {
                        0.0..FLOAT_EPSILON => 0.0,
                        _ => player_weighted_minerals / total_weighted_minerals,
                    },
                },
            );

            player_raw_kpi_data.insert(
                KPIComponent::Supply,
                PlayerRawKPIData {
                    source_value: player_supply_count,
                    weighted_value: player_supply_count,
                    mission_total_weighted_value: total_supply_count,
                    raw_index: match total_supply_count {
                        0.0..FLOAT_EPSILON => 0.0,
                        _ => -player_supply_count / total_supply_count,
                    },
                },
            );

            raw_kpi_data.insert(player_info.player_id, player_raw_kpi_data);
        }

        let result = MissionKPICachedInfo {
            mission_id: mission_info.mission_info.id,
            damage_map,
            kill_map,
            resource_map: resource_map.clone(),
            total_damage_map,
            total_kill_map,
            total_resource_map,
            player_id_to_kpi_character,
            raw_kpi_data,
        };

        let elapsed = begin.elapsed();

        debug!(
            "generated cached kpi info for {} in {:?}",
            mission_info.mission_info.id, elapsed
        );

        (result, elapsed)
    }

    pub fn from_redis_all(
        db_conn: &mut PgConnection,
        redis_conn: &mut redis::Connection,
        entity_blacklist_set: &HashSet<String>,
        entity_combine: &HashMap<String, String>,
        weapon_combine: &HashMap<String, String>,
        character_id_to_game_id: &HashMap<i16, String>,
        player_id_to_name: &HashMap<i16, String>,
        scout_special_player_set: &HashSet<String>,
        kpi_config: &KPIConfig,
    ) -> Result<Vec<Self>, ()> {
        let begin = Instant::now();
        let mission_list = MissionCachedInfo::get_cached_all(
            db_conn,
            redis_conn,
            entity_blacklist_set,
            entity_combine,
            weapon_combine,
        )?;

        let load_from_redis_elapsed = begin.elapsed();
        let begin = Instant::now();

        let mut result = Vec::with_capacity(mission_list.len());

        for mission_info in &mission_list {
            let generated = Self::generate(
                &mission_info,
                character_id_to_game_id,
                player_id_to_name,
                scout_special_player_set,
                kpi_config,
            )
            .0;
            result.push(generated);
        }

        let generate_elapsed = begin.elapsed();

        info!("generated {} cached mission kpi info from redis in {:?}(total) = {:?}(load_from_redis) + {:?}(generate)", result.len(), load_from_redis_elapsed + generate_elapsed, load_from_redis_elapsed, generate_elapsed);

        Ok(result)
    }

    pub fn get_cached(
        db_conn: &mut PgConnection,
        redis_conn: &mut redis::Connection,
        entity_blacklist_set: &HashSet<String>,
        entity_combine: &HashMap<String, String>,
        weapon_combine: &HashMap<String, String>,
        character_id_to_game_id: &HashMap<i16, String>,
        player_id_to_name: &HashMap<i16, String>,
        scout_special_player_set: &HashSet<String>,
        kpi_config: &KPIConfig,
        mission_id: i32,
    ) -> Result<Self, ()> {
        let cached_bytes: Option<Vec<u8>> = redis_conn
            .get(format!("mission_kpi_raw:{}", mission_id))
            .ok();

        let cached_content = match cached_bytes {
            Some(x) => {
                let decoded: MissionKPICachedInfo = match rmp_serde::from_read(&x[..]) {
                    Ok(x) => x,
                    Err(e) => {
                        error!("cannot decode cached bytes: {}", e);
                        return Err(());
                    }
                };

                decoded
            }
            None => {
                let mission = MissionCachedInfo::get_cached(
                    db_conn,
                    redis_conn,
                    entity_blacklist_set,
                    entity_combine,
                    weapon_combine,
                    mission_id,
                )?;
                let generated = Self::generate(
                    &mission,
                    character_id_to_game_id,
                    player_id_to_name,
                    scout_special_player_set,
                    kpi_config,
                )
                .0;
                let serialized = rmp_serde::to_vec(&generated).unwrap();
                match redis_conn.set(format!("mission_kpi_raw:{}", mission_id), serialized) {
                    Ok(()) => generated,
                    Err(e) => {
                        error!("cannot write data to redis: {}", e);
                        return Err(());
                    }
                }
            }
        };

        Ok(cached_content)
    }

    pub fn get_cached_all(
        db_conn: &mut PgConnection,
        redis_conn: &mut redis::Connection,
        entity_blacklist_set: &HashSet<String>,
        entity_combine: &HashMap<String, String>,
        weapon_combine: &HashMap<String, String>,
        character_id_to_game_id: &HashMap<i16, String>,
        player_id_to_name: &HashMap<i16, String>,
        scout_special_player_set: &HashSet<String>,
        kpi_config: &KPIConfig,
    ) -> Result<Vec<Self>, ()> {
        let mission_list = MissionCachedInfo::get_cached_all(
            db_conn,
            redis_conn,
            entity_blacklist_set,
            entity_combine,
            weapon_combine,
        )?;

        let mut result = Vec::with_capacity(mission_list.len());

        for mission_info in &mission_list {
            let mission_id = mission_info.mission_info.id;
            let cached_bytes: Option<Vec<u8>> = redis_conn
                .get(format!("mission_kpi_raw:{}", mission_id))
                .ok();

            let cached_content = match cached_bytes {
                Some(x) => {
                    let decoded: MissionKPICachedInfo = match rmp_serde::from_read(&x[..]) {
                        Ok(x) => x,
                        Err(e) => {
                            error!("cannot decode cached bytes: {}", e);
                            return Err(());
                        }
                    };

                    decoded
                }
                None => {
                    let generated = Self::generate(
                        &mission_info,
                        character_id_to_game_id,
                        player_id_to_name,
                        scout_special_player_set,
                        kpi_config,
                    )
                    .0;
                    let serialized = rmp_serde::to_vec(&generated).unwrap();
                    match redis_conn.set(format!("mission_kpi_raw:{}", mission_id), serialized) {
                        Ok(()) => generated,
                        Err(e) => {
                            error!("cannot write data to redis: {}", e);
                            return Err(());
                        }
                    }
                }
            };

            result.push(cached_content);
        }

        Ok(result)
    }
}
