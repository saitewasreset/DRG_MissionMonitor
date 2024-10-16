pub mod character;
pub mod entity;
pub mod general;
pub mod weapon;
use actix_web::web;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
pub struct FriendlyFireData {
    #[serde(rename = "gameCount")]
    pub game_count: i32,
    pub damage: f64,
    pub show: bool,
}

impl Default for FriendlyFireData {
    fn default() -> Self {
        FriendlyFireData {
            game_count: 0,
            damage: 0.0,
            show: false,
        }
    }
}

#[derive(Serialize)]
pub struct PlayerFriendlyFireInfo {
    pub cause: HashMap<String, FriendlyFireData>,
    pub take: HashMap<String, FriendlyFireData>,
}

impl Default for PlayerFriendlyFireInfo {
    fn default() -> Self {
        PlayerFriendlyFireInfo {
            cause: HashMap::new(),
            take: HashMap::new(),
        }
    }
}

#[derive(Serialize)]
pub struct PlayerDamageInfo {
    pub damage: HashMap<String, f64>,
    pub kill: HashMap<String, i32>,
    pub ff: PlayerFriendlyFireInfo,
    #[serde(rename = "averageSupplyCount")]
    pub average_supply_count: f64,
    #[serde(rename = "validGameCount")]
    pub valid_game_count: i32,
}

impl Default for PlayerDamageInfo {
    fn default() -> Self {
        PlayerDamageInfo {
            damage: HashMap::new(),
            kill: HashMap::new(),
            ff: PlayerFriendlyFireInfo::default(),
            average_supply_count: 0.0,
            valid_game_count: 0,
        }
    }
}

#[derive(Serialize)]
pub struct OverallDamageInfo {
    pub info: HashMap<String, PlayerDamageInfo>,
    #[serde(rename = "prevInfo")]
    pub prev_info: HashMap<String, PlayerDamageInfo>,
    #[serde(rename = "entityMapping")]
    pub entity_mapping: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct DamagePack {
    pub taker_id: i16,
    pub taker_type: i16,
    pub weapon_id: i16,
    pub total_amount: f64,
}

#[derive(Serialize, Deserialize)]
pub struct KillPack {
    pub taker_id: i16,
    pub taker_name: String,
    pub total_amount: i32,
}

#[derive(Serialize, Deserialize)]
pub struct WeaponPack {
    pub weapon_id: i16,
    // 含友伤
    pub total_amount: f64,
    pub detail: HashMap<String, DamagePack>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct SupplyPack {
    pub ammo: f64,
    pub health: f64,
}

#[derive(Serialize)]
pub struct WeaponDamageInfo {
    // 不含友伤
    pub damage: f64,
    #[serde(rename = "friendlyFire")]
    pub friendly_fire: f64,
    #[serde(rename = "heroGameId")]
    pub hero_game_id: String,
    #[serde(rename = "mappedName")]
    pub mapped_name: String,
    #[serde(rename = "validGameCount")]
    pub valid_game_count: i32,
}

#[derive(Serialize)]
pub struct CharacterFriendlyFireInfo {
    cause: f64,
    take: f64,
}

#[derive(Serialize)]
pub struct CharacterDamageInfo {
    pub damage: f64,
    #[serde(rename = "friendlyFire")]
    pub friendly_fire: CharacterFriendlyFireInfo,
    #[serde(rename = "playerIndex")]
    pub player_index: f64,
    #[serde(rename = "mappedName")]
    pub mapped_name: String,
}

#[derive(Serialize)]
pub struct EntityDamageInfo {
    pub damage: HashMap<String, f64>,
    pub kill: HashMap<String, i32>,
    #[serde(rename = "entityMapping")]
    pub entity_mapping: HashMap<String, String>,
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(general::get_overall_damage_info);
    cfg.service(weapon::get_damage_weapon);
    cfg.service(character::get_damage_character);
    cfg.service(entity::get_damage_entity);
}
