-- Usuários e sessões
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL,
    username_lower TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL,
    email_lower TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    locale TEXT NOT NULL DEFAULT 'pt-BR',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash BYTEA NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS sessions_user_idx ON sessions(user_id);
CREATE INDEX IF NOT EXISTS sessions_expires_idx ON sessions(expires_at);

-- Decks
CREATE TABLE IF NOT EXISTS decks (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    cards JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Partidas
CREATE TABLE IF NOT EXISTS matches (
    id UUID PRIMARY KEY,
    player_blue UUID REFERENCES users(id) ON DELETE SET NULL,
    player_red UUID REFERENCES users(id) ON DELETE SET NULL,
    winner UUID REFERENCES users(id) ON DELETE SET NULL,
    result_blue TEXT NOT NULL,
    result_red TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    blue_crowns SMALLINT NOT NULL,
    red_crowns SMALLINT NOT NULL,
    blue_damage REAL NOT NULL,
    red_damage REAL NOT NULL,
    end_reason TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ NOT NULL
);

-- Replays (log de snapshots comprimido com gzip)
CREATE TABLE IF NOT EXISTS replays (
    id UUID PRIMARY KEY,
    match_id UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    data BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS replays_user_idx ON replays(user_id, created_at DESC);
