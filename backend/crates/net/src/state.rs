use std::sync::Arc;

use royale_auth::AuthService;
use royale_db::Db;
use royale_defs::{Catalog, Definitions};
use royale_shared::GameCommand;
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

use crate::hub::ConnectionHub;
use crate::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct NetConfig {
    pub cookie_secure: bool,
    pub session_days: i64,
    pub allowed_origins: Vec<String>,
}

impl Default for NetConfig {
    fn default() -> Self {
        Self {
            cookie_secure: false,
            session_days: royale_shared::SESSION_DAYS,
            allowed_origins: vec![
                "http://localhost:5173".into(),
                "http://127.0.0.1:5173".into(),
            ],
        }
    }
}

pub struct NetDeps {
    pub db: Db,
    pub auth: AuthService,
    pub defs: Arc<Definitions>,
    pub catalog: Catalog,
    pub game_tx: UnboundedSender<GameCommand>,
    pub default_deck: Vec<String>,
    pub config: NetConfig,
}

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<AppInner>,
}

pub struct AppInner {
    pub db: Db,
    pub auth: AuthService,
    pub defs: Arc<Definitions>,
    pub catalog: Catalog,
    pub content: Value,
    pub game_tx: UnboundedSender<GameCommand>,
    pub default_deck: Vec<String>,
    pub config: NetConfig,
    pub limiter: RateLimiter,
    pub hub: ConnectionHub,
}

impl AppState {
    pub fn new(deps: NetDeps) -> Self {
        let content = serde_json::json!({
            "version": royale_shared::CONTENT_VERSION,
            "catalog": deps.catalog,
            "definitions": deps.defs.as_ref(),
        });
        Self {
            inner: Arc::new(AppInner {
                db: deps.db,
                auth: deps.auth,
                defs: deps.defs,
                catalog: deps.catalog,
                content,
                game_tx: deps.game_tx,
                default_deck: deps.default_deck,
                config: deps.config,
                limiter: RateLimiter::new(),
                hub: ConnectionHub::new(),
            }),
        }
    }

    pub fn db(&self) -> &Db {
        &self.inner.db
    }

    pub fn auth(&self) -> &AuthService {
        &self.inner.auth
    }

    pub fn defs(&self) -> &Definitions {
        &self.inner.defs
    }

    pub fn default_deck(&self) -> &[String] {
        &self.inner.default_deck
    }

    pub fn cookie_secure(&self) -> bool {
        self.inner.config.cookie_secure
    }

    pub fn session_days(&self) -> i64 {
        self.inner.config.session_days
    }

    pub fn hub(&self) -> &ConnectionHub {
        &self.inner.hub
    }

    pub fn limiter(&self) -> &RateLimiter {
        &self.inner.limiter
    }

    pub fn game_tx(&self) -> &UnboundedSender<GameCommand> {
        &self.inner.game_tx
    }
}
