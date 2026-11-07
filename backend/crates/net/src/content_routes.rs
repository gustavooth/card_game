use axum::Json;
use axum::extract::State;

use crate::state::AppState;

pub async fn health(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db_ok = state.db().ping().await.is_ok();
    Json(serde_json::json!({
        "ok": db_ok,
        "online": state.hub().online(),
        "version": royale_shared::CONTENT_VERSION,
    }))
}

pub async fn content(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(state.inner.content.clone())
}
