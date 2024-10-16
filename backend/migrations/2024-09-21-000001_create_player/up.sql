CREATE TABLE player (
    id SMALLSERIAL PRIMARY KEY,
    player_name TEXT NOT NULL,
    friend BOOLEAN NOT NULL
);

CREATE UNIQUE INDEX uni_idx_player_player_name ON player (player_name);