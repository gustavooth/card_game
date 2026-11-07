//! HTTP REST + WebSocket, extração de sessão e rate limiting.

mod auth_routes;
mod content_routes;
mod deck_routes;
mod error;
mod extractors;
mod hub;
mod profile_routes;
mod rate_limit;
mod replay_routes;
mod state;
mod ws;

pub use error::ApiError;
pub use hub::ConnectionHub;
pub use rate_limit::RateLimiter;
pub use state::{AppState, NetConfig, NetDeps};

use axum::Router;
use axum::routing::{get, post, put};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn router(state: AppState) -> Router {
    let origins: Vec<axum::http::HeaderValue> = state
        .inner
        .config
        .allowed_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    Router::new()
        .route("/api/health", get(content_routes::health))
        .route("/api/auth/register", post(auth_routes::register))
        .route("/api/auth/login", post(auth_routes::login))
        .route("/api/auth/logout", post(auth_routes::logout))
        .route("/api/auth/me", get(auth_routes::me))
        .route("/api/content", get(content_routes::content))
        .route(
            "/api/deck",
            get(deck_routes::get_deck).put(deck_routes::put_deck),
        )
        .route("/api/profile", get(profile_routes::profile))
        .route("/api/profile/locale", put(profile_routes::set_locale))
        .route("/api/replays", get(replay_routes::list))
        .route("/api/replays/{id}", get(replay_routes::download))
        .route("/ws", get(ws::upgrade))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            rate_limit::global_middleware,
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(origins))
                .allow_credentials(true)
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::ACCEPT,
                    axum::http::header::AUTHORIZATION,
                ]),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
