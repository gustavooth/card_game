CREATE INDEX IF NOT EXISTS matches_blue_idx ON matches(player_blue, ended_at DESC);
CREATE INDEX IF NOT EXISTS matches_red_idx ON matches(player_red, ended_at DESC);
CREATE INDEX IF NOT EXISTS replays_match_idx ON replays(match_id);
