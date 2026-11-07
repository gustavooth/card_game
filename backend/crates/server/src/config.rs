use std::net::SocketAddr;

use anyhow::{Context, Result};

pub struct Config {
    pub bind: SocketAddr,
    pub database_url: String,
    pub db_pool_size: usize,
    pub cookie_secure: bool,
    pub session_days: i64,
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let bind = env("BIND_ADDR", "0.0.0.0:8090")
            .parse()
            .context("BIND_ADDR inválido")?;
        let database_url = env(
            "DATABASE_URL",
            "postgres://royale:royale@localhost:5433/royale",
        );
        let db_pool_size = env("DB_POOL_SIZE", "16")
            .parse()
            .context("DB_POOL_SIZE inválido")?;
        let cookie_secure = env("COOKIE_SECURE", "false")
            .parse()
            .context("COOKIE_SECURE inválido")?;
        let session_days = env("SESSION_DAYS", "30")
            .parse()
            .context("SESSION_DAYS inválido")?;
        let allowed_origins = env(
            "ALLOWED_ORIGINS",
            "http://localhost:5173,http://127.0.0.1:5173",
        )
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(str::to_string)
        .collect();
        Ok(Self {
            bind,
            database_url,
            db_pool_size,
            cookie_secure,
            session_days,
            allowed_origins,
        })
    }
}

fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
