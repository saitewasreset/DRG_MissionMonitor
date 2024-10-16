CREATE TABLE kill_info (
    id SERIAL PRIMARY KEY,
    mission_id INTEGER NOT NULL REFERENCES mission,
    time SMALLINT NOT NULL,
    player_id SMALLINT NOT NULL REFERENCES player,
    entity_id SMALLINT NOT NULL REFERENCES entity
);