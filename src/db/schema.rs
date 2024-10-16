// @generated automatically by Diesel CLI.

diesel::table! {
    character (id) {
        id -> Int2,
        character_game_id -> Text,
    }
}

diesel::table! {
    damage_info (id) {
        id -> Int4,
        mission_id -> Int4,
        time -> Int2,
        damage -> Float8,
        causer_id -> Int2,
        taker_id -> Int2,
        weapon_id -> Int2,
        causer_type -> Int2,
        taker_type -> Int2,
    }
}

diesel::table! {
    entity (id) {
        id -> Int2,
        entity_game_id -> Text,
    }
}

diesel::table! {
    kill_info (id) {
        id -> Int4,
        mission_id -> Int4,
        time -> Int2,
        player_id -> Int2,
        entity_id -> Int2,
    }
}

diesel::table! {
    mission (id) {
        id -> Int4,
        begin_timestamp -> Int8,
        mission_time -> Int2,
        mission_type_id -> Int2,
        hazard_id -> Int2,
        result -> Int2,
        reward_credit -> Float8,
        total_supply_count -> Int2,
    }
}

diesel::table! {
    mission_invalid (id) {
        id -> Int4,
        mission_id -> Int4,
        reason -> Text,
    }
}

diesel::table! {
    mission_type (id) {
        id -> Int2,
        mission_type_game_id -> Text,
    }
}

diesel::table! {
    player (id) {
        id -> Int2,
        player_name -> Text,
        friend -> Bool,
    }
}

diesel::table! {
    player_info (id) {
        id -> Int4,
        mission_id -> Int4,
        player_id -> Int2,
        character_id -> Int2,
        player_rank -> Int2,
        character_rank -> Int2,
        character_promotion -> Int2,
        present_time -> Int2,
        kill_num -> Int2,
        revive_num -> Int2,
        death_num -> Int2,
        gold_mined -> Float8,
        minerals_mined -> Float8,
        player_escaped -> Bool,
    }
}

diesel::table! {
    resource (id) {
        id -> Int2,
        resource_game_id -> Text,
    }
}

diesel::table! {
    resource_info (id) {
        id -> Int4,
        mission_id -> Int4,
        player_id -> Int2,
        time -> Int2,
        resource_id -> Int2,
        amount -> Float8,
    }
}

diesel::table! {
    supply_info (id) {
        id -> Int4,
        mission_id -> Int4,
        player_id -> Int2,
        time -> Int2,
        ammo -> Float8,
        health -> Float8,
    }
}

diesel::table! {
    weapon (id) {
        id -> Int2,
        weapon_game_id -> Text,
    }
}

diesel::joinable!(damage_info -> mission (mission_id));
diesel::joinable!(kill_info -> entity (entity_id));
diesel::joinable!(kill_info -> mission (mission_id));
diesel::joinable!(kill_info -> player (player_id));
diesel::joinable!(mission -> mission_type (mission_type_id));
diesel::joinable!(mission_invalid -> mission (mission_id));
diesel::joinable!(player_info -> character (character_id));
diesel::joinable!(player_info -> mission (mission_id));
diesel::joinable!(player_info -> player (player_id));
diesel::joinable!(resource_info -> mission (mission_id));
diesel::joinable!(resource_info -> player (player_id));
diesel::joinable!(resource_info -> resource (resource_id));
diesel::joinable!(supply_info -> mission (mission_id));
diesel::joinable!(supply_info -> player (player_id));

diesel::allow_tables_to_appear_in_same_query!(
    character,
    damage_info,
    entity,
    kill_info,
    mission,
    mission_invalid,
    mission_type,
    player,
    player_info,
    resource,
    resource_info,
    supply_info,
    weapon,
);
