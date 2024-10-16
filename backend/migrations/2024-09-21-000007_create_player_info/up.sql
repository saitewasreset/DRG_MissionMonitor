CREATE TABLE player_info (
    id SERIAL PRIMARY KEY,
    mission_id INTEGER NOT NULL REFERENCES mission,
    player_id SMALLINT NOT NULL REFERENCES player,
    character_id SMALLINT NOT NULL REFERENCES character,
    player_rank SMALLINT NOT NULL,
    character_rank SMALLINT NOT NULL,
    character_promotion SMALLINT NOT NULL,
    present_time SMALLINT NOT NULL,
    kill_num SMALLINT NOT NULL,
    revive_num SMALLINT NOT NULL,
    death_num SMALLINT NOT NULL,
    gold_mined DOUBLE PRECISION NOT NULL,
    minerals_mined DOUBLE PRECISION NOT NULL,
    player_escaped BOOLEAN NOT NULL
);