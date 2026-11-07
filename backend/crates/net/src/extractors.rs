use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use royale_db::models::User;

use crate::error::ApiError;
use crate::state::AppState;

pub struct AuthUser {
    pub user: User,
    pub token: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::Unauthorized)?;
        let token = jar
            .get(royale_shared::SESSION_COOKIE)
            .map(|cookie| cookie.value().to_string())
            .filter(|token| !token.is_empty())
            .ok_or(ApiError::Unauthorized)?;
        let Some((_session, user)) = state
            .auth()
            .authenticate(&token)
            .await
            .map_err(ApiError::from)?
        else {
            return Err(ApiError::Unauthorized);
        };
        Ok(AuthUser { user, token })
    }
}

pub fn session_cookie(token: String, secure: bool, days: i64) -> Cookie<'static> {
    Cookie::build((royale_shared::SESSION_COOKIE, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(cookie::time::Duration::days(days))
        .build()
}

pub fn clear_session_cookie() -> Cookie<'static> {
    Cookie::build((royale_shared::SESSION_COOKIE, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(cookie::time::Duration::ZERO)
        .build()
}
