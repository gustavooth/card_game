use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use royale_auth::AuthError;
use royale_db::DbError;

#[derive(Debug)]
pub enum ApiError {
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict(String),
    BadRequest(String),
    RateLimited(&'static str),
    Internal(String),
}

impl ApiError {
    pub fn internal(error: impl std::fmt::Display) -> Self {
        tracing::error!(%error, "erro interno");
        Self::Internal("Erro interno do servidor".to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Sessão inválida ou expirada".to_string(),
            ),
            ApiError::Forbidden => (
                StatusCode::FORBIDDEN,
                "forbidden",
                "Acesso negado".to_string(),
            ),
            ApiError::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "Recurso não encontrado".to_string(),
            ),
            ApiError::Conflict(message) => (StatusCode::CONFLICT, "conflict", message),
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, "bad_request", message),
            ApiError::RateLimited(message) => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                message.to_string(),
            ),
            ApiError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, "internal", message),
        };
        (
            status,
            Json(serde_json::json!({ "code": code, "message": message })),
        )
            .into_response()
    }
}

impl From<DbError> for ApiError {
    fn from(error: DbError) -> Self {
        match error {
            DbError::Conflict(message) => ApiError::Conflict(message.to_string()),
            DbError::NotFound => ApiError::NotFound,
            other => ApiError::internal(other),
        }
    }
}

impl From<AuthError> for ApiError {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::InvalidCredentials => ApiError::Unauthorized,
            AuthError::InvalidInput(message) => ApiError::BadRequest(message.to_string()),
            AuthError::Db(error) => error.into(),
            AuthError::Hash(error) => ApiError::internal(error),
        }
    }
}
