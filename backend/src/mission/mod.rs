use actix_web::web;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{damage::SupplyPack, db::models::Mission};
pub mod load;
pub mod mission;
pub mod mission_list;

#[derive(Serialize, Deserialize)]
pub struct APIMission {
    pub id: i32,
    pub begin_timestamp: i64,
    pub mission_time: i16,
    pub mission_type: String,
    pub hazard_id: i16,
    pub result: i16,
    pub reward_credit: f64,
    pub total_supply_count: i16,
}

impl APIMission {
    fn from_mission(mission_type_map: &HashMap<i16, String>, mission: Mission) -> Self {
        let mission_type = match mission_type_map.get(&mission.mission_type_id) {
            Some(mission_type) => mission_type.clone(),
            None => mission.mission_type_id.to_string(),
        };

        APIMission {
            id: mission.id,
            begin_timestamp: mission.begin_timestamp,
            mission_time: mission.mission_time,
            mission_type,
            hazard_id: mission.hazard_id,
            result: mission.result,
            reward_credit: mission.reward_credit,
            total_supply_count: mission.total_supply_count,
        }
    }
}

#[derive(Serialize)]
pub struct MissionInfo {
    #[serde(rename = "missionId")]
    pub mission_id: i32,
    #[serde(rename = "beginTimestamp")]
    pub begin_timestamp: i64,
    #[serde(rename = "missionTime")]
    pub mission_time: i16,
    #[serde(rename = "missionTypeId")]
    pub mission_type_id: String,
    #[serde(rename = "hazardId")]
    pub hazard_id: i16,
    #[serde(rename = "missionResult")]
    pub mission_result: i16,
    #[serde(rename = "rewardCredit")]
    pub reward_credit: f64,
    #[serde(rename = "missionInvalid")]
    pub mission_invalid: bool,
    #[serde(rename = "missionInvalidReason")]
    pub mission_invalid_reason: String,
}

#[derive(Serialize)]
pub struct MissionList {
    #[serde(rename = "missionInfo")]
    pub mission_info: Vec<MissionInfo>,
    #[serde(rename = "missionTypeMapping")]
    pub mission_type_mapping: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct MissionGeneralInfo {
    #[serde(rename = "missionId")]
    pub mission_id: i32,
    #[serde(rename = "missionBeginTimestamp")]
    pub mission_begin_timestamp: i64,
    #[serde(rename = "missionInvalid")]
    pub mission_invalid: bool,
    #[serde(rename = "missionInvalidReason")]
    pub mission_invalid_reason: String,
}

#[derive(Serialize)]
pub struct MissionGeneralPlayerInfo {
    #[serde(rename = "characterGameId")]
    pub character_game_id: String,
    #[serde(rename = "playerRank")]
    pub player_rank: i16,
    #[serde(rename = "characterRank")]
    pub character_rank: i16,
    #[serde(rename = "characterPromotion")]
    pub character_promotion: i16,
    #[serde(rename = "presentTime")]
    pub present_time: i16,
    #[serde(rename = "reviveNum")]
    pub revive_num: i16,
    #[serde(rename = "deathNum")]
    pub death_num: i16,
    #[serde(rename = "playerEscaped")]
    pub player_escaped: bool,
}

#[derive(Serialize)]
pub struct MissionGeneralData {
    #[serde(rename = "beginTimeStamp")]
    pub begin_timestamp: i64,
    #[serde(rename = "hazardId")]
    pub hazard_id: i16,
    #[serde(rename = "missionResult")]
    pub mission_result: i16,
    #[serde(rename = "missionTime")]
    pub mission_time: i16,
    #[serde(rename = "missionTypeId")]
    pub mission_type_id: String,
    #[serde(rename = "playerInfo")]
    pub player_info: HashMap<String, MissionGeneralPlayerInfo>,
    #[serde(rename = "rewardCredit")]
    pub reward_credit: f64,
    #[serde(rename = "totalDamage")]
    pub total_damage: f64,
    #[serde(rename = "totalKill")]
    pub total_kill: i32,
    #[serde(rename = "totalMinerals")]
    pub total_minerals: f64,
    #[serde(rename = "totalNitra")]
    pub total_nitra: f64,
    #[serde(rename = "totalSupplyCount")]
    pub total_supply_count: i16,
}

#[derive(Serialize)]
pub struct PlayerFriendlyFireInfo {
    cause: HashMap<String, f64>,
    take: HashMap<String, f64>,
}

#[derive(Serialize)]
pub struct PlayerDamageInfo {
    pub damage: HashMap<String, f64>,
    pub kill: HashMap<String, i32>,
    pub ff: PlayerFriendlyFireInfo,
    #[serde(rename = "supplyCount")]
    pub supply_count: i16,
}

#[derive(Serialize)]
pub struct MissionDamageInfo {
    pub info: HashMap<String, PlayerDamageInfo>,
    #[serde(rename = "entityMapping")]
    pub entity_mapping: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct MissionWeaponDamageInfo {
    pub damage: f64,
    #[serde(rename = "friendlyFire")]
    pub friendly_fire: f64,
    #[serde(rename = "characterGameId")]
    pub character_game_id: String,
    #[serde(rename = "mappedName")]
    pub mapped_name: String,
}

#[derive(Serialize)]
pub struct PlayerResourceData {
    pub resource: HashMap<String, f64>,
    pub supply: Vec<SupplyPack>,
}

#[derive(Serialize)]
pub struct MissionResourceInfo {
    data: HashMap<String, PlayerResourceData>,
    #[serde(rename = "resourceMapping")]
    resource_mapping: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct MissionKPIComponent {
    pub name: String,
    #[serde(rename = "sourceValue")]
    pub source_value: f64,
    #[serde(rename = "weightedValue")]
    pub weighted_value: f64,
    #[serde(rename = "missionTotalWeightedValue")]
    pub mission_total_weighted_value: f64,
    #[serde(rename = "rawIndex")]
    pub raw_index: f64,
    #[serde(rename = "correctedIndex")]
    pub corrected_index: f64,
    #[serde(rename = "transformedIndex")]
    pub transformed_index: f64,
    pub weight: f64,
}

#[derive(Serialize)]
pub struct MissionKPIInfo {
    #[serde(rename = "playerName")]
    pub player_name: String,
    #[serde(rename = "kpiCharacterType")]
    pub kpi_character_type: String,
    #[serde(rename = "weightedKill")]
    pub weighted_kill: f64,
    #[serde(rename = "weightedDamage")]
    pub weighted_damage: f64,
    #[serde(rename = "priorityDamage")]
    pub priority_damage: f64,
    #[serde(rename = "reviveNum")]
    pub revive_num: f64,
    #[serde(rename = "deathNum")]
    pub death_num: f64,
    #[serde(rename = "friendlyFire")]
    pub friendly_fire: f64,
    pub nitra: f64,
    #[serde(rename = "supplyCount")]
    pub supply_count: f64,
    #[serde(rename = "weightedResource")]
    pub weighted_resource: f64,
    pub component: Vec<MissionKPIComponent>,
    #[serde(rename = "missionKPI")]
    pub mission_kpi: f64,
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(load::load_mission);
    cfg.service(mission_list::get_api_mission_list);
    cfg.service(mission_list::get_mission_list);

    cfg.service(mission::get_general_info);
    cfg.service(mission::get_mission_general);
    cfg.service(mission::get_mission_damage);
    cfg.service(mission::get_mission_weapon_damage);
    cfg.service(mission::get_mission_resource_info);
    cfg.service(mission::get_player_character);
    cfg.service(mission::get_mission_kpi);
}
