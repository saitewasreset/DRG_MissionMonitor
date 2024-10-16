CREATE TABLE mission_invalid (
    id SERIAL PRIMARY KEY,
    mission_id INTEGER NOT NULL REFERENCES mission,
    reason TEXT NOT NULL
);