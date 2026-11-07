//! Hash de senhas (Argon2id), tokens opacos de sessão e validação de credenciais.

use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use chrono::{Duration, Utc};
use rand::RngCore;
use royale_db::models::{SessionRow, User};
use royale_db::{Db, DbError};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("credenciais inválidas")]
    InvalidCredentials,
    #[error("entrada inválida: {0}")]
    InvalidInput(&'static str),
    #[error("banco: {0}")]
    Db(#[from] DbError),
    #[error("hash: {0}")]
    Hash(String),
}

#[derive(Clone)]
pub struct AuthService {
    db: Db,
    session_days: i64,
}

pub struct AuthResult {
    pub user: User,
    pub token: String,
}

impl AuthService {
    pub fn new(db: Db, session_days: i64) -> Self {
        Self { db, session_days }
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResult, AuthError> {
        let username = username.trim();
        let email = email.trim();
        validate_username(username)?;
        validate_email(email)?;
        validate_password(password)?;

        let password = password.to_string();
        let hash = tokio::task::spawn_blocking(move || hash_password(&password))
            .await
            .map_err(|error| AuthError::Hash(error.to_string()))??;

        let user = self
            .db
            .create_user(royale_db::queries::CreateUser {
                username: username.to_string(),
                email: email.to_string(),
                password_hash: hash,
                locale: "pt-BR".to_string(),
            })
            .await?;
        let token = self.create_session(user.id).await?;
        Ok(AuthResult { user, token })
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<AuthResult, AuthError> {
        let Some((user, hash)) = self.db.find_user_by_email(email.trim()).await? else {
            return Err(AuthError::InvalidCredentials);
        };
        let password = password.to_string();
        let valid = tokio::task::spawn_blocking(move || verify_password(&password, &hash))
            .await
            .map_err(|error| AuthError::Hash(error.to_string()))??;
        if !valid {
            return Err(AuthError::InvalidCredentials);
        }
        let token = self.create_session(user.id).await?;
        Ok(AuthResult { user, token })
    }

    async fn create_session(&self, user_id: uuid::Uuid) -> Result<String, AuthError> {
        let mut raw = [0u8; 32];
        OsRng.fill_bytes(&mut raw);
        let token = hex::encode(raw);
        let hash = hash_token(&token);
        let expires_at = Utc::now() + Duration::days(self.session_days);
        self.db.create_session(user_id, &hash, expires_at).await?;
        Ok(token)
    }

    pub async fn authenticate(&self, token: &str) -> Result<Option<(SessionRow, User)>, AuthError> {
        if token.len() < 16 {
            return Ok(None);
        }
        let hash = hash_token(token);
        let found = self.db.find_session(&hash).await?;
        if let Some((session, user)) = &found {
            let _ = self.db.touch_session(session.id).await;
            let _ = user;
        }
        Ok(found)
    }

    pub async fn logout(&self, token: &str) -> Result<(), AuthError> {
        let hash = hash_token(token);
        self.db.delete_session(&hash).await?;
        Ok(())
    }

    pub async fn cleanup_sessions(&self) {
        if let Err(error) = self.db.prune_expired_sessions().await {
            tracing::warn!(%error, "falha ao limpar sessões expiradas");
        }
    }
}

pub fn hash_token(token: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hasher.finalize().to_vec()
}

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| AuthError::Hash(error.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed = PasswordHash::new(hash).map_err(|error| AuthError::Hash(error.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

fn validate_username(username: &str) -> Result<(), AuthError> {
    let length = username.chars().count();
    if !(3..=20).contains(&length) {
        return Err(AuthError::InvalidInput(
            "nome deve ter entre 3 e 20 caracteres",
        ));
    }
    if !username
        .chars()
        .all(|character| character.is_alphanumeric() || character == '_' || character == '-')
    {
        return Err(AuthError::InvalidInput(
            "nome deve conter apenas letras, números, _ ou -",
        ));
    }
    Ok(())
}

fn validate_email(email: &str) -> Result<(), AuthError> {
    let valid = email.len() <= 254
        && email.split('@').count() == 2
        && email
            .split('@')
            .nth(1)
            .is_some_and(|domain| domain.contains('.') && !domain.starts_with('.'));
    if !valid {
        return Err(AuthError::InvalidInput("e-mail inválido"));
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), AuthError> {
    if password.chars().count() < 8 {
        return Err(AuthError::InvalidInput(
            "senha deve ter ao menos 8 caracteres",
        ));
    }
    if password.chars().count() > 128 {
        return Err(AuthError::InvalidInput("senha muito longa"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_roundtrip() {
        let hash = hash_password("uma-senha-forte").expect("hash");
        assert!(verify_password("uma-senha-forte", &hash).expect("verify"));
        assert!(!verify_password("senha-errada", &hash).expect("verify"));
    }

    #[test]
    fn session_token_hash_is_deterministic() {
        let token = "0123456789abcdef0123456789abcdef";
        assert_eq!(hash_token(token), hash_token(token));
        assert_ne!(hash_token(token), hash_token("outro-token"));
        assert_eq!(hash_token(token).len(), 32);
    }

    #[test]
    fn input_validation_rejects_short_values() {
        assert!(validate_username("ab").is_err());
        assert!(validate_email("sem-arroba").is_err());
        assert!(validate_password("curta").is_err());
        assert!(validate_username("jogador_1").is_ok());
    }
}
