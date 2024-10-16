pub mod character;
pub mod general;
pub mod mission_type;
pub mod player;
use std::collections::HashMap;

use actix_web::web;
use serde::Serialize;

#[derive(Serialize)]
pub struct DeltaData<T: Serialize> {
    prev: T,
    recent: T,
    total: T,
}

#[derive(Serialize)]
pub struct GeneralInfo {
    #[serde(rename = "gameCount")]
    pub game_count: i32,
    #[serde(rename = "validRate")]
    pub valid_rate: f64,
    #[serde(rename = "totalMissionTime")]
    pub total_mission_time: i64,
    #[serde(rename = "averageMissionTime")]
    pub average_mission_time: DeltaData<i16>,
    #[serde(rename = "uniquePlayerCount")]
    pub unique_player_count: i32,
    #[serde(rename = "openRoomRate")]
    pub open_room_rate: DeltaData<f64>,
    #[serde(rename = "passRate")]
    pub pass_rate: DeltaData<f64>,
    #[serde(rename = "averageDifficulty")]
    pub average_difficulty: DeltaData<f64>,
    #[serde(rename = "averageKillNum")]
    pub average_kill_num: DeltaData<i16>,
    #[serde(rename = "averageDamage")]
    pub average_damage: DeltaData<f64>,
    #[serde(rename = "averageDeathNumPerPlayer")]
    pub average_death_num_per_player: DeltaData<f64>,
    #[serde(rename = "averageMineralsMined")]
    pub average_minerals_mined: DeltaData<f64>,
    #[serde(rename = "averageSupplyCountPerPlayer")]
    pub average_supply_count_per_player: DeltaData<f64>,
    #[serde(rename = "averageRewardCredit")]
    pub average_reward_credit: DeltaData<f64>,
}

#[derive(Serialize)]
pub struct MissionTypeData {
    #[serde(rename = "averageDifficulty")]
    pub average_difficulty: f64,
    #[serde(rename = "averageMissionTime")]
    pub average_mission_time: f64,
    #[serde(rename = "averageRewardCredit")]
    pub average_reward_credit: f64,
    #[serde(rename = "creditPerMinute")]
    pub credit_per_minute: f64,
    #[serde(rename = "missionCount")]
    pub mission_count: i32,
    #[serde(rename = "passRate")]
    pub pass_rate: f64,
}

#[derive(Serialize)]
pub struct MissionTypeInfo {
    #[serde(rename = "missionTypeMap")]
    pub mission_type_map: HashMap<String, String>,
    // mission_game_id -> MissionTypeData
    #[serde(rename = "missionTypeData")]
    pub mission_type_data: HashMap<String, MissionTypeData>,
}

#[derive(Serialize)]
pub struct PlayerData {
    #[serde(rename = "averageDeathNum")]
    pub average_death_num: f64,
    #[serde(rename = "averageMineralsMined")]
    pub average_minerals_mined: f64,
    #[serde(rename = "averageReviveNum")]
    pub average_revive_num: f64,
    #[serde(rename = "averageSupplyCount")]
    pub average_supply_count: f64,
    #[serde(rename = "averageSupplyEfficiency")]
    pub average_supply_efficiency: f64,
    #[serde(rename = "characterInfo")]
    pub character_info: HashMap<String, i32>,
    #[serde(rename = "validMissionCount")]
    pub valid_mission_count: i32,
}

#[derive(Serialize)]
pub struct PlayerInfo {
    #[serde(rename = "characterMap")]
    // character_game_id -> name
    pub character_map: HashMap<String, String>,
    #[serde(rename = "playerData")]
    // player_name -> PlayerData
    pub player_data: HashMap<String, PlayerData>,
    #[serde(rename = "prevPlayerData")]
    pub prev_player_data: HashMap<String, PlayerData>,
}

#[derive(Serialize)]
pub struct CharacterGeneralData {
    #[serde(rename = "playerIndex")]
    pub player_index: f64,
    #[serde(rename = "reviveNum")]
    pub revive_num: f64,
    #[serde(rename = "deathNum")]
    pub death_num: f64,
    #[serde(rename = "mineralsMined")]
    pub minerals_mined: f64,
    #[serde(rename = "supplyCount")]
    pub supply_count: f64,
    #[serde(rename = "supplyEfficiency")]
    pub supply_efficiency: f64,
}

#[derive(Serialize)]
pub struct CharacterGeneralInfo {
    #[serde(rename = "characterMapping")]
    pub character_mapping: HashMap<String, String>,
    #[serde(rename = "characterData")]
    pub character_data: HashMap<String, CharacterGeneralData>,
}

#[derive(Serialize)]
pub struct CharacterChoiceInfo {
    #[serde(rename = "characterChoiceCount")]
    pub character_choice_count: HashMap<String, i32>,
    #[serde(rename = "characterMapping")]
    pub character_mapping: HashMap<String, String>,
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(general::get_general);
    cfg.service(mission_type::get_mission_type);
    cfg.service(player::get_player);
    cfg.service(character::get_character_general_info);
    cfg.service(character::get_character_choice_info);
}
