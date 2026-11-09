use std::net::SocketAddr;

use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;

use crate::error::ApiError;
use crate::extractors::{AuthUser, clear_session_cookie, session_cookie};
use crate::rate_limit::client_ip;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub email: String,
    pub password: String,
    pub password_confirmation: String,
    #[serde(default)]
    pub locale: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

pub async fn register(
    State(state): State<AppState>,
    connect: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<RegisterPayload>,
) -> Result<Response, ApiError> {
    if payload.password != payload.password_confirmation {
        return Err(ApiError::BadRequest("As senhas não conferem".into()));
    }
    let ip = client_ip(&headers, Some(connect));
    // Limite generoso por IP + e-mail: protege contra abuso sem bloquear
    // tentativas legítimas de corrigir dados do formulário.
    if !state.limiter().check(
        &format!("register:{ip}:{}", payload.email.trim().to_lowercase()),
        20.0,
        20.0 / 600.0,
    ) {
        return Err(ApiError::RateLimited(
            "Muitas tentativas de registro. Aguarde alguns minutos.",
        ));
    }
    let locale = match payload.locale.as_deref() {
        Some("en") => "en",
        _ => "pt-BR",
    };
    let result = state
        .auth()
        .register(&payload.username, &payload.email, &payload.password)
        .await?;
    if locale != result.user.locale {
        let _ = state.db().set_user_locale(result.user.id, locale).await;
    }
    let _ = state
        .db()
        .set_deck(result.user.id, state.default_deck())
        .await;
    let cookie = session_cookie(result.token, state.cookie_secure(), state.session_days());
    let user = serde_json::json!({ "user": result.user });
    Ok((jar.add(cookie), Json(user)).into_response())
}

pub async fn login(
    State(state): State<AppState>,
    connect: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<LoginPayload>,
) -> Result<Response, ApiError> {
    let ip = client_ip(&headers, Some(connect));
    if !state.limiter().check(
        &format!("login:{ip}:{}", payload.email.trim().to_lowercase()),
        30.0,
        0.5,
    ) {
        return Err(ApiError::RateLimited(
            "Muitas tentativas de login. Aguarde um instante.",
        ));
    }
    let result = state
        .auth()
        .login(&payload.email, &payload.password)
        .await?;
    let cookie = session_cookie(result.token, state.cookie_secure(), state.session_days());
    let user = serde_json::json!({ "user": result.user });
    Ok((jar.add(cookie), Json(user)).into_response())
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    user: AuthUser,
) -> Result<Response, ApiError> {
    state.auth().logout(&user.token).await?;
    Ok((
        jar.add(clear_session_cookie()),
        Json(serde_json::json!({ "ok": true })),
    )
        .into_response())
}

pub async fn me(user: AuthUser) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({ "user": user.user })))
}
