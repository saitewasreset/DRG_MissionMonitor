CREATE TABLE mission (
    mission_id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    begin_timestamp BIGINT UNSIGNED NOT NULL,
    mission_time INT UNSIGNED NOT NULL,
    mission_type_id TINYINT UNSIGNED NOT NULL,
    hazard_id TINYINT UNSIGNED NOT NULL,
    result TINYINT UNSIGNED NOT NULL,
    reward_credit DOUBLE NOT NULL,
    total_supply_count INT UNSIGNED NOT NULL
);

CREATE TABLE mission_type (
    mission_type_id TINYINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    mission_type_game_id VARCHAR(64) NOT NULL
);

CREATE TABLE player_info (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    mission_id INT UNSIGNED NOT NULL,
    player_id SMALLINT UNSIGNED NOT NULL,
    hero_id TINYINT UNSIGNED NOT NULL,
    player_rank SMALLINT UNSIGNED NOT NULL,
    character_rank TINYINT UNSIGNED NOT NULL,
    character_promotion SMALLINT UNSIGNED NOT NULL,
    present_time INT UNSIGNED NOT NULL,
    kill_num SMALLINT UNSIGNED NOT NULL,
    revive_num TINYINT UNSIGNED NOT NULL,
    death_num TINYINT UNSIGNED NOT NULL,
    gold_mined DOUBLE NOT NULL,
    minerals_mined DOUBLE NOT NULL,
    player_escaped TINYINT UNSIGNED NOT NULL,
    present_at_end TINYINT UNSIGNED NOT NULL
);

CREATE TABLE player (
    player_id SMALLINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    player_name VARCHAR(256),
    friend TINYINT UNSIGNED NOT NULL
)
CHARACTER SET 'utf8mb4'
COLLATE 'utf8mb4_general_ci';

CREATE TABLE hero (
    hero_id TINYINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    hero_game_id VARCHAR(16) NOT NULL
);

CREATE TABLE damage (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    mission_id INT UNSIGNED NOT NULL,
    time INT UNSIGNED NOT NULL,
    damage DOUBLE,
    causer_id INT UNSIGNED NOT NULL,
    taker_id INT UNSIGNED NOT NULL,
    weapon_id TINYINT UNSIGNED NOT NULL,
    causer_type TINYINT UNSIGNED NOT NULL,
    taker_type TINYINT UNSIGNED NOT NULL
);

CREATE TABLE entity (
    entity_id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    entity_game_id VARCHAR(128)
);

CREATE TABLE weapon (
    weapon_id TINYINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    weapon_game_id VARCHAR(64)
);


CREATE TABLE kill_info (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    mission_id INT UNSIGNED NOT NULL,
    time INT UNSIGNED NOT NULL,
    causer_id INT UNSIGNED NOT NULL,
    killed_entity_id INT UNSIGNED NOT NULL
);

CREATE TABLE resource_info (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    mission_id INT UNSIGNED NOT NULL,
    time INT UNSIGNED NOT NULL,
    player_id SMALLINT UNSIGNED NOT NULL,
    resource_id SMALLINT UNSIGNED NOT NULL,
    amount DOUBLE NOT NULL
);

CREATE TABLE resource (
    resource_id SMALLINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    resource_game_id VARCHAR(64) NOT NULL
);

CREATE TABLE supply_info (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    mission_id INT UNSIGNED NOT NULL,
    time INT UNSIGNED NOT NULL,
    player_id SMALLINT UNSIGNED NOT NULL,
    ammo DOUBLE,
    health DOUBLE
);

CREATE TABLE mission_invalid (
    mission_id INT UNSIGNED PRIMARY KEY,
    reason VARCHAR(256)
)
CHARACTER SET 'utf8mb4'
COLLATE 'utf8mb4_general_ci';

CREATE INDEX idx_begin_timestamp_mission ON mission(begin_timestamp);
CREATE INDEX idx_mission_type_id_mission ON mission(mission_type_id);

CREATE INDEX idx_mission_type_game_id_mission_type ON mission_type(mission_type_game_id);

CREATE INDEX idx_mission_id_player_info ON player_info(mission_id);
CREATE INDEX idx_player_id_player_info ON player_info(player_id);
CREATE INDEX idx_hero_id_player_info ON player_info(hero_id);

CREATE INDEX idx_mission_id_damage ON damage(mission_id);
CREATE INDEX idx_taker_id_damage ON damage(taker_id);
CREATE INDEX idx_causer_id_damage ON damage(causer_id);
CREATE INDEX idx_weapon_id_damage ON damage(weapon_id);
CREATE INDEX idx_taker_type_damage ON damage(taker_type);
CREATE INDEX idx_causer_type_damage ON damage(causer_type);
CREATE INDEX idx_causer_taker_type_damage ON damage(causer_type, taker_type);

CREATE INDEX idx_mission_id_kill_info ON kill_info(mission_id);
CREATE INDEX idx_causer_id_kill_info ON kill_info(causer_id);
CREATE INDEX idx_killed_entity_id_kill_info ON kill_info(killed_entity_id);

CREATE INDEX idx_mission_id_resource_info ON resource_info(mission_id);
CREATE INDEX idx_resource_id_resource_info ON resource_info(resource_id);

CREATE INDEX idx_mission_id_supply_info ON supply_info(mission_id);
CREATE INDEX idx_player_id_supply_info ON supply_info(player_id);

INSERT INTO entity (entity_id, entity_game_id) VALUES (1, 'Unknown');
INSERT INTO weapon (weapon_id, weapon_game_id) VALUES (1, 'Unknown');