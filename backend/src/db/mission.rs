use super::mission_log::LogContent;
use super::mission_log::LogDamageInfo;
use super::mission_log::LogKillInfo;
use super::mission_log::LogMissionInfo;
use super::mission_log::LogPlayerInfo;
use super::mission_log::LogResourceInfo;
use super::mission_log::LogSupplyInfo;
use super::models::Character;
use super::models::Entity;
use super::models::Mission;
use super::models::MissionType;
use super::models::Player;
use super::models::Resource;
use super::models::Weapon;
use super::schema::*;
use super::DbError;

use diesel::{insert_into, prelude::*};
use std::collections::HashMap;

#[derive(Insertable)]
#[diesel(table_name = super::schema::mission)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMission {
    pub begin_timestamp: i64,
    pub mission_time: i16,
    pub mission_type_id: i16,
    pub hazard_id: i16,
    pub result: i16,
    pub reward_credit: f64,
    pub total_supply_count: i16,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::player_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPlayerInfo {
    pub mission_id: i32,
    pub player_id: i16,
    pub character_id: i16,
    pub player_rank: i16,
    pub character_rank: i16,
    pub character_promotion: i16,
    pub present_time: i16,
    pub kill_num: i16,
    pub revive_num: i16,
    pub death_num: i16,
    pub gold_mined: f64,
    pub minerals_mined: f64,
    pub player_escaped: bool,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::damage_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDamageInfo {
    pub mission_id: i32,
    pub time: i16,
    pub damage: f64,
    pub causer_id: i16,
    pub taker_id: i16,
    pub weapon_id: i16,
    pub causer_type: i16,
    pub taker_type: i16,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::kill_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewKillInfo {
    pub mission_id: i32,
    pub time: i16,
    pub player_id: i16,
    pub entity_id: i16,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::resource_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewResourceInfo {
    pub mission_id: i32,
    pub time: i16,
    pub player_id: i16,
    pub resource_id: i16,
    pub amount: f64,
}

#[derive(Insertable)]
#[diesel(table_name = super::schema::supply_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSupplyInfo {
    pub mission_id: i32,
    pub time: i16,
    pub player_id: i16,
    pub ammo: f64,
    pub health: f64,
}

impl NewMission {
    pub fn from_mission_log(
        mission_type_map: &mut HashMap<String, i16>,
        db: &mut PgConnection,
        mission_log: LogMissionInfo,
    ) -> Result<NewMission, DbError> {
        let mission_type_game_id = mission_log.mission_type_id;

        let mission_type_id = match mission_type_map.get(&mission_type_game_id) {
            None => {
                let mission_type_id = insert_into(mission_type::table)
                    .values(mission_type::mission_type_game_id.eq(&mission_type_game_id))
                    .get_result::<(i16, String)>(db)
                    .map_err(|e| {
                        DbError::UnexpectedError(format!(
                            "from_mission_log: db error while inserting mission type: {}",
                            e
                        ))
                    })?
                    .0;

                mission_type_map.insert(mission_type_game_id.clone(), mission_type_id);
                mission_type_id
            }
            Some(mission_type_id) => *mission_type_id,
        };

        Ok(NewMission {
            begin_timestamp: mission_log.begin_timestamp,
            mission_time: mission_log.mission_time,
            mission_type_id,
            hazard_id: mission_log.hazard_id,
            result: mission_log.result,
            reward_credit: mission_log.reward_credit,
            total_supply_count: mission_log.total_supply_count,
        })
    }
}

impl NewPlayerInfo {
    pub fn from_player_info_log(
        player_id_map: &mut HashMap<String, i16>,
        character_id_map: &mut HashMap<String, i16>,
        db: &mut PgConnection,
        mission_id: i32,
        player_info_log: LogPlayerInfo,
    ) -> Result<NewPlayerInfo, DbError> {
        let player_id = match player_id_map.get(&player_info_log.player_name) {
            None => {
                let player_id = insert_into(player::table)
                    .values((
                        player::player_name.eq(&player_info_log.player_name),
                        player::friend.eq(false),
                    ))
                    .get_result::<(i16, String, bool)>(db)
                    .map_err(|e| {
                        DbError::UnexpectedError(format!(
                            "from_player_info_log: db error while inserting player: {}",
                            e
                        ))
                    })?
                    .0;

                player_id_map.insert(player_info_log.player_name.clone(), player_id);
                player_id
            }
            Some(player_id) => *player_id,
        };

        let character_id = match character_id_map.get(&player_info_log.character) {
            None => {
                let character_id = insert_into(character::table)
                    .values((character::character_game_id.eq(&player_info_log.character),))
                    .get_result::<(i16, String)>(db)
                    .map_err(|e| {
                        DbError::UnexpectedError(format!(
                            "from_player_info_log: db error while inserting character: {}",
                            e
                        ))
                    })?
                    .0;

                character_id_map.insert(player_info_log.character.clone(), character_id);
                character_id
            }
            Some(character_id) => *character_id,
        };

        Ok(NewPlayerInfo {
            mission_id,
            player_id,
            character_id,
            player_rank: player_info_log.player_rank,
            character_rank: player_info_log.character_rank,
            character_promotion: player_info_log.character_promotion,
            present_time: player_info_log.total_present_time,
            kill_num: player_info_log.kill_num,
            revive_num: player_info_log.revive_num,
            death_num: player_info_log.death_num,
            gold_mined: player_info_log.gold_mined,
            minerals_mined: player_info_log.minerals_mined,
            player_escaped: player_info_log.player_escaped,
        })
    }
}

impl NewDamageInfo {
    pub fn from_damage_info_log(
        mission_id: i32,
        player_id_map: &mut HashMap<String, i16>,
        entity_map: &mut HashMap<String, i16>,
        weapon_map: &mut HashMap<String, i16>,
        db: &mut PgConnection,
        damage_info_log: LogDamageInfo,
    ) -> Result<NewDamageInfo, DbError> {
        let causer_id = match damage_info_log.causer_type {
            1 => match player_id_map.get(&damage_info_log.causer) {
                None => {
                    let player_id = insert_into(player::table)
                        .values((
                            player::player_name.eq(&damage_info_log.causer),
                            player::friend.eq(false),
                        ))
                        .get_result::<(i16, String, bool)>(db)
                        .map_err(|e| {
                            DbError::UnexpectedError(format!(
                                "from_damage_info_log: db error while inserting player: {}",
                                e
                            ))
                        })?
                        .0;

                    player_id_map.insert(damage_info_log.causer.clone(), player_id);
                    player_id
                }
                Some(causer_id) => *causer_id,
            },
            _ => match entity_map.get(&damage_info_log.causer) {
                None => {
                    let causer_id = insert_into(entity::table)
                        .values(entity::entity_game_id.eq(&damage_info_log.causer))
                        .get_result::<(i16, String)>(db)
                        .map_err(|e| {
                            DbError::UnexpectedError(format!(
                                "from_damage_info_log: db error while inserting entity: {}",
                                e
                            ))
                        })?
                        .0;

                    entity_map.insert(damage_info_log.causer.clone(), causer_id);
                    causer_id
                }
                Some(causer_id) => *causer_id,
            },
        };

        let taker_id = match damage_info_log.taker_type {
            1 => match player_id_map.get(&damage_info_log.taker) {
                None => {
                    let player_id = insert_into(player::table)
                        .values((
                            player::player_name.eq(&damage_info_log.taker),
                            player::friend.eq(false),
                        ))
                        .get_result::<(i16, String, bool)>(db)
                        .map_err(|e| {
                            DbError::UnexpectedError(format!(
                                "from_damage_info_log: db error while inserting player: {}",
                                e
                            ))
                        })?
                        .0;

                    player_id_map.insert(damage_info_log.taker, player_id);
                    player_id
                }
                Some(taker_id) => *taker_id,
            },
            _ => match entity_map.get(&damage_info_log.taker) {
                None => {
                    let entity_id = insert_into(entity::table)
                        .values(entity::entity_game_id.eq(&damage_info_log.taker))
                        .get_result::<(i16, String)>(db)
                        .map_err(|e| {
                            DbError::UnexpectedError(format!(
                                "from_damage_info_log: db error while inserting entity: {}",
                                e
                            ))
                        })?
                        .0;
                    entity_map.insert(damage_info_log.taker.clone(), entity_id);
                    entity_id
                }
                Some(taker_id) => *taker_id,
            },
        };

        let weapon_id = match weapon_map.get(&damage_info_log.weapon) {
            None => {
                let weapon_id = insert_into(weapon::table)
                    .values(weapon::weapon_game_id.eq(&damage_info_log.weapon))
                    .get_result::<(i16, String)>(db)
                    .map_err(|e| {
                        DbError::UnexpectedError(format!(
                            "from_damage_info_log: db error while inserting weapon: {}",
                            e
                        ))
                    })?
                    .0;
                weapon_map.insert(damage_info_log.weapon.clone(), weapon_id);
                weapon_id
            }
            Some(weapon_id) => *weapon_id,
        };

        Ok(NewDamageInfo {
            mission_id,
            time: damage_info_log.mission_time,
            damage: damage_info_log.damage,
            causer_id,
            taker_id,
            weapon_id,
            causer_type: damage_info_log.causer_type,
            taker_type: damage_info_log.taker_type,
        })
    }
}

impl NewKillInfo {
    pub fn from_kill_info_log(
        mission_id: i32,
        player_id_map: &mut HashMap<String, i16>,
        entity_map: &mut HashMap<String, i16>,
        db: &mut PgConnection,
        kill_info_log: LogKillInfo,
    ) -> Result<NewKillInfo, DbError> {
        let player_id = match player_id_map.get(&kill_info_log.player_name) {
            None => {
                let player_id = insert_into(player::table)
                    .values((
                        player::player_name.eq(&kill_info_log.player_name),
                        player::friend.eq(false),
                    ))
                    .get_result::<(i16, String, bool)>(db)
                    .map_err(|e| {
                        DbError::UnexpectedError(format!(
                            "from_kill_info_log: db error while inserting player: {}",
                            e
                        ))
                    })?
                    .0;
                player_id_map.insert(kill_info_log.player_name.clone(), player_id);
                player_id
            }
            Some(player_id) => *player_id,
        };

        let entity_id = match entity_map.get(&kill_info_log.killed_entity) {
            None => {
                let entity_id = insert_into(entity::table)
                    .values((entity::entity_game_id.eq(&kill_info_log.killed_entity),))
                    .get_result::<(i16, String)>(db)
                    .map_err(|e| {
                        DbError::UnexpectedError(format!(
                            "from_kill_info_log: db error while inserting entity: {}",
                            e
                        ))
                    })?
                    .0;
                entity_map.insert(kill_info_log.killed_entity.clone(), entity_id);
                entity_id
            }
            Some(entity_id) => *entity_id,
        };

        Ok(NewKillInfo {
            mission_id,
            time: kill_info_log.mission_time,
            player_id,
            entity_id,
        })
    }
}

impl NewResourceInfo {
    pub fn from_resource_info_log(
        mission_id: i32,
        player_id_map: &mut HashMap<String, i16>,
        resource_map: &mut HashMap<String, i16>,
        db: &mut PgConnection,
        resource_info_log: LogResourceInfo,
    ) -> Result<NewResourceInfo, DbError> {
        let player_id = match player_id_map.get(&resource_info_log.player_name) {
            None => {
                let player_id = insert_into(player::table)
                    .values((
                        player::player_name.eq(&resource_info_log.player_name),
                        player::friend.eq(false),
                    ))
                    .get_result::<(i16, String, bool)>(db)
                    .map_err(|e| {
                        DbError::UnexpectedError(format!(
                            "from_resource_info_log: db error while inserting player: {}",
                            e
                        ))
                    })?
                    .0;
                player_id_map.insert(resource_info_log.player_name.clone(), player_id);
                player_id
            }
            Some(player_id) => *player_id,
        };

        let resource_id = match resource_map.get(&resource_info_log.resource) {
            None => {
                let resource_id = insert_into(resource::table)
                    .values((resource::resource_game_id.eq(&resource_info_log.resource),))
                    .get_result::<(i16, String)>(db)
                    .map_err(|e| {
                        DbError::UnexpectedError(format!(
                            "from_resource_info_log: db error while inserting resource: {}",
                            e
                        ))
                    })?
                    .0;
                resource_map.insert(resource_info_log.resource.clone(), resource_id);
                resource_id
            }
            Some(resource_id) => *resource_id,
        };

        Ok(NewResourceInfo {
            mission_id,
            time: resource_info_log.mission_time,
            player_id,
            resource_id,
            amount: resource_info_log.amount,
        })
    }
}

impl NewSupplyInfo {
    pub fn from_supply_info_log(
        mission_id: i32,
        player_id_map: &mut HashMap<String, i16>,
        db: &mut PgConnection,
        supply_info_log: LogSupplyInfo,
    ) -> Result<NewSupplyInfo, DbError> {
        let player_id = match player_id_map.get(&supply_info_log.player_name) {
            None => {
                let player_id = insert_into(player::table)
                    .values((
                        player::player_name.eq(&supply_info_log.player_name),
                        player::friend.eq(false),
                    ))
                    .get_result::<(i16, String, bool)>(db)
                    .map_err(|e| {
                        DbError::UnexpectedError(format!(
                            "from_supply_info_log: db error while inserting player: {}",
                            e
                        ))
                    })?
                    .0;
                player_id_map.insert(supply_info_log.player_name.clone(), player_id);
                player_id
            }
            Some(player_id) => *player_id,
        };

        Ok(NewSupplyInfo {
            mission_id,
            time: supply_info_log.mission_time,
            player_id,
            ammo: supply_info_log.ammo,
            health: supply_info_log.health,
        })
    }
}

pub fn load_mission(log: LogContent, db: &mut PgConnection) -> Result<(), DbError> {
    let player_list: Vec<Player> = player::table.load(db).map_err(|e| {
        DbError::UnexpectedError(format!(
            "load_mission: db error while fetching player: {}",
            e
        ))
    })?;

    let mut player_id_map = HashMap::with_capacity(player_list.len());

    player_list.into_iter().for_each(|p| {
        player_id_map.insert(p.player_name, p.id);
    });

    let entity_list: Vec<Entity> = entity::table.load(db).map_err(|e| {
        DbError::UnexpectedError(format!(
            "load_mission: db error while fetching entity: {}",
            e
        ))
    })?;

    let mut entity_map = HashMap::with_capacity(entity_list.len());

    entity_list.into_iter().for_each(|entity| {
        entity_map.insert(entity.entity_game_id, entity.id);
    });

    let character_list: Vec<Character> = character::table.load(db).map_err(|e| {
        DbError::UnexpectedError(format!(
            "load_mission: db error while fetching character: {}",
            e
        ))
    })?;

    let mut character_map = HashMap::with_capacity(character_list.len());

    character_list.into_iter().for_each(|character| {
        character_map.insert(character.character_game_id, character.id);
    });

    let resource_list: Vec<Resource> = resource::table.load(db).map_err(|e| {
        DbError::UnexpectedError(format!(
            "load_mission: db error while fetching resource: {}",
            e
        ))
    })?;

    let mut resource_map = HashMap::with_capacity(resource_list.len());

    resource_list.into_iter().for_each(|resource| {
        resource_map.insert(resource.resource_game_id, resource.id);
    });

    let weapon_list: Vec<Weapon> = weapon::table.load(db).map_err(|e| {
        DbError::UnexpectedError(format!(
            "load_mission: db error while fetching weapon: {}",
            e
        ))
    })?;

    let mut weapon_map = HashMap::with_capacity(weapon_list.len());

    weapon_list.into_iter().for_each(|weapon| {
        weapon_map.insert(weapon.weapon_game_id, weapon.id);
    });

    let mission_type_list: Vec<MissionType> = mission_type::table.load(db).map_err(|e| {
        DbError::UnexpectedError(format!(
            "load_mission: db error while fetching mission type: {}",
            e
        ))
    })?;

    let mut mission_type_map = HashMap::with_capacity(mission_type_list.len());

    mission_type_list.into_iter().for_each(|mt| {
        mission_type_map.insert(mt.mission_type_game_id, mt.id);
    });

    let new_mission_info =
        NewMission::from_mission_log(&mut mission_type_map, db, log.mission_info)?;

    let inserted_mission: Mission = insert_into(mission::table)
        .values(&new_mission_info)
        .get_result(db)
        .map_err(|e| {
            DbError::UnexpectedError(format!(
                "load_mission: db error while fetching inserted mission for mission id: {}",
                e
            ))
        })?;

    let inserted_mission_id = inserted_mission.id;

    let mut new_player_info_list = Vec::with_capacity(log.player_info.len());

    for source_info in log.player_info {
        new_player_info_list.push(NewPlayerInfo::from_player_info_log(
            &mut player_id_map,
            &mut character_map,
            db,
            inserted_mission_id,
            source_info,
        )?);
    }

    insert_into(player_info::table)
        .values(&new_player_info_list)
        .execute(db)
        .map_err(|e| {
            DbError::UnexpectedError(format!("db error while inserting player info: {}", e))
        })?;

    let mut new_damage_info_list = Vec::with_capacity(log.damage_info.len());

    for source_info in log.damage_info {
        new_damage_info_list.push(NewDamageInfo::from_damage_info_log(
            inserted_mission_id,
            &mut player_id_map,
            &mut entity_map,
            &mut weapon_map,
            db,
            source_info,
        )?);
    }

    for new_damage_info_chunk in new_damage_info_list.chunks(4096) {
        insert_into(damage_info::table)
            .values(new_damage_info_chunk)
            .execute(db)
            .map_err(|e| {
                DbError::UnexpectedError(format!("db error while inserting damage info: {}", e))
            })?;
    }

    let mut new_kill_info_list = Vec::with_capacity(log.kill_info.len());

    for source_info in log.kill_info {
        new_kill_info_list.push(NewKillInfo::from_kill_info_log(
            inserted_mission_id,
            &mut player_id_map,
            &mut entity_map,
            db,
            source_info,
        )?);
    }

    for new_kill_info_chunk in new_kill_info_list.chunks(4096) {
        insert_into(kill_info::table)
            .values(new_kill_info_chunk)
            .execute(db)
            .map_err(|e| {
                DbError::UnexpectedError(format!("db error while inserting kill info: {}", e))
            })?;
    }

    let mut new_resource_info_list = Vec::with_capacity(log.resource_info.len());

    for source_info in log.resource_info {
        new_resource_info_list.push(NewResourceInfo::from_resource_info_log(
            inserted_mission_id,
            &mut player_id_map,
            &mut resource_map,
            db,
            source_info,
        )?);
    }

    insert_into(resource_info::table)
        .values(&new_resource_info_list)
        .execute(db)
        .map_err(|e| {
            DbError::UnexpectedError(format!("db error while inserting resource info: {}", e))
        })?;

    let mut new_supply_info_list = Vec::with_capacity(log.supply_info.len());

    for source_info in log.supply_info {
        new_supply_info_list.push(NewSupplyInfo::from_supply_info_log(
            inserted_mission_id,
            &mut player_id_map,
            db,
            source_info,
        )?);
    }

    insert_into(supply_info::table)
        .values(&new_supply_info_list)
        .execute(db)
        .map_err(|e| {
            DbError::UnexpectedError(format!("db error while inserting supply info: {}", e))
        })?;

    Ok(())
}
