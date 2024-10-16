CREATE TABLE resource_info (
    id SERIAL PRIMARY KEY,
    mission_id INTEGER NOT NULL REFERENCES mission,
    player_id SMALLINT NOT NULL REFERENCES player,
    time SMALLINT NOT NULL,
    resource_id SMALLINT NOT NULL REFERENCES resource,
    amount DOUBLE PRECISION NOT NULL
);