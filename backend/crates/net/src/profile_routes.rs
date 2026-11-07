use axum::Json;
use axum::extract::State;
use serde::Deserialize;

use crate::error::ApiError;
use crate::extractors::AuthUser;
use crate::state::AppState;

pub async fn profile(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let stats = state.db().player_stats(user.user.id).await?;
    let recent = state.db().recent_matches(user.user.id, 10).await?;
    Ok(Json(serde_json::json!({
        "user": user.user,
        "stats": stats,
        "recent": recent,
    })))
}

#[derive(Deserialize)]
pub struct LocalePayload {
    pub locale: String,
}

pub async fn set_locale(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<LocalePayload>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let locale = match payload.locale.as_str() {
        "pt-BR" => "pt-BR",
        "en" => "en",
        _ => return Err(ApiError::BadRequest("Idioma inválido".into())),
    };
    state.db().set_user_locale(user.user.id, locale).await?;
    Ok(Json(serde_json::json!({ "locale": locale })))
}
