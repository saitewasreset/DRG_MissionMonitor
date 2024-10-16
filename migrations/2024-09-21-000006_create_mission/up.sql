CREATE TABLE mission (
    id SERIAL PRIMARY KEY,
    begin_timestamp BIGINT NOT NULL,
    mission_time SMALLINT NOT NULL,
    mission_type_id SMALLINT NOT NULL REFERENCES mission_type,
    hazard_id SMALLINT NOT NULL,
    result SMALLINT NOT NULL,
    reward_credit DOUBLE PRECISION NOT NULL,
    total_supply_count SMALLINT NOT NULL
);