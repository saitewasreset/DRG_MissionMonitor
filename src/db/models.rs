use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Clone)]
#[diesel(table_name = super::schema::mission)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Mission {
    pub id: i32,
    pub begin_timestamp: i64,
    pub mission_time: i16,
    pub mission_type_id: i16,
    pub hazard_id: i16,
    pub result: i16,
    pub reward_credit: f64,
    pub total_supply_count: i16,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(Mission))]
#[diesel(belongs_to(Player))]
#[diesel(belongs_to(Character))]
#[diesel(table_name = super::schema::player_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlayerInfo {
    pub id: i32,
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

#[derive(Queryable, Selectable, Identifiable, Associations, Clone)]
#[diesel(belongs_to(Mission))]
#[diesel(belongs_to(Weapon))]
#[diesel(table_name = super::schema::damage_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DamageInfo {
    pub id: i32,
    pub mission_id: i32,
    pub time: i16,
    pub damage: f64,
    pub causer_id: i16,
    pub taker_id: i16,
    pub weapon_id: i16,
    pub causer_type: i16,
    pub taker_type: i16,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Clone)]
#[diesel(belongs_to(Mission))]
#[diesel(belongs_to(Player))]
#[diesel(belongs_to(Entity))]
#[diesel(table_name = super::schema::kill_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct KillInfo {
    pub id: i32,
    pub mission_id: i32,
    pub time: i16,
    pub player_id: i16,
    pub entity_id: i16,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Clone)]
#[diesel(belongs_to(Mission))]
#[diesel(belongs_to(Player))]
#[diesel(belongs_to(Resource))]
#[diesel(table_name = super::schema::resource_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ResourceInfo {
    pub id: i32,
    pub mission_id: i32,
    pub player_id: i16,
    pub time: i16,
    pub resource_id: i16,
    pub amount: f64,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Clone)]
#[diesel(belongs_to(Mission))]
#[diesel(belongs_to(Player))]
#[diesel(table_name = super::schema::supply_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SupplyInfo {
    pub id: i32,
    pub mission_id: i32,
    pub player_id: i16,
    pub time: i16,
    pub ammo: f64,
    pub health: f64,
}

#[derive(Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = super::schema::player)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Player {
    pub id: i16,
    pub player_name: String,
    pub friend: bool,
}

#[derive(Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = super::schema::entity)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Entity {
    pub id: i16,
    pub entity_game_id: String,
}

#[derive(Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = super::schema::character)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Character {
    pub id: i16,
    pub character_game_id: String,
}

#[derive(Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = super::schema::resource)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Resource {
    pub id: i16,
    pub resource_game_id: String,
}

#[derive(Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = super::schema::weapon)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Weapon {
    pub id: i16,
    pub weapon_game_id: String,
}

#[derive(Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = super::schema::mission_type)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MissionType {
    pub id: i16,
    pub mission_type_game_id: String,
}

#[derive(Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = super::schema::mission_invalid)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MissionInvalid {
    pub id: i32,
    pub mission_id: i32,
    pub reason: String,
}
