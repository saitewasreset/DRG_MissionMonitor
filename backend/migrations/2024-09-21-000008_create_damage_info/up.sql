CREATE TABLE damage_info (
    id SERIAL PRIMARY KEY,
    mission_id INTEGER NOT NULL REFERENCES mission,
    time SMALLINT NOT NULL,
    damage DOUBLE PRECISION NOT NULL,
    causer_id SMALLINT NOT NULL,
    taker_id SMALLINT NOT NULL,
    weapon_id SMALLINT NOT NULL,
    causer_type SMALLINT NOT NULL,
    taker_type SMALLINT NOT NULL
);
