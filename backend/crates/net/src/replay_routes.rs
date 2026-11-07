use axum::Json;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::error::ApiError;
use crate::extractors::AuthUser;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let replays = state.db().list_replays(user.user.id).await?;
    Ok(Json(serde_json::json!({ "replays": replays })))
}

pub async fn download(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    if !state
        .limiter()
        .check(&format!("replay:{}", user.user.id), 60.0, 1.0)
    {
        return Err(ApiError::RateLimited("Muitos downloads de replay."));
    }
    let data = state
        .db()
        .get_replay_data(user.user.id, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (header::CONTENT_ENCODING, "gzip"),
            (header::CACHE_CONTROL, "private, max-age=3600"),
        ],
        data,
    )
        .into_response())
}
