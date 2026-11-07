use axum::Json;
use axum::extract::State;
use serde::Deserialize;

use crate::error::ApiError;
use crate::extractors::AuthUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct DeckPayload {
    pub cards: Vec<String>,
}

pub async fn get_deck(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let stored = state.db().get_deck(user.user.id).await?;
    let cards = match stored {
        Some(cards) if cards.len() == 8 => cards,
        _ => {
            let default = state.default_deck().to_vec();
            let _ = state.db().set_deck(user.user.id, &default).await;
            default
        }
    };
    Ok(Json(serde_json::json!({ "cards": cards })))
}

pub async fn put_deck(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<DeckPayload>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !state
        .limiter()
        .check(&format!("deck:{}", user.user.id), 10.0, 10.0 / 60.0)
    {
        return Err(ApiError::RateLimited("Muitas alterações de deck."));
    }
    royale_core::deck_indices(state.defs(), &state.inner.catalog, &payload.cards)
        .map_err(|message| ApiError::BadRequest(message.to_string()))?;
    state.db().set_deck(user.user.id, &payload.cards).await?;
    Ok(Json(serde_json::json!({ "cards": payload.cards })))
}
