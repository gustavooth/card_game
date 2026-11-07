use chrono::Utc;
use tokio_postgres::error::SqlState;
use uuid::Uuid;

use crate::models::User;
use crate::{Db, DbError};

pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub locale: String,
}

pub struct CreatedUser {
    pub user: User,
}

impl Db {
    pub async fn create_user(&self, input: CreateUser) -> Result<User, DbError> {
        let client = self.pool.get().await?;
        let id = Uuid::new_v4();
        let now = Utc::now();
        let username_lower = input.username.to_lowercase();
        let email_lower = input.email.to_lowercase();
        let row = client
            .query_one(
                "INSERT INTO users (id, username, username_lower, email, email_lower, password_hash, locale, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                 RETURNING id, username, email, locale, created_at",
                &[
                    &id,
                    &input.username,
                    &username_lower,
                    &input.email,
                    &email_lower,
                    &input.password_hash,
                    &input.locale,
                    &now,
                ],
            )
            .await;
        match row {
            Ok(row) => Ok(User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                locale: row.get("locale"),
                created_at: row.get("created_at"),
            }),
            Err(error) if error.code() == Some(&SqlState::UNIQUE_VIOLATION) => {
                Err(DbError::Conflict("nome ou e-mail já cadastrado"))
            }
            Err(error) => Err(error.into()),
        }
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<(User, String)>, DbError> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT id, username, email, locale, created_at, password_hash
                 FROM users WHERE email_lower = $1",
                &[&email.to_lowercase()],
            )
            .await?;
        Ok(row.map(|row| {
            (
                User {
                    id: row.get("id"),
                    username: row.get("username"),
                    email: row.get("email"),
                    locale: row.get("locale"),
                    created_at: row.get("created_at"),
                },
                row.get("password_hash"),
            )
        }))
    }

    pub async fn find_user_by_id(&self, id: Uuid) -> Result<Option<User>, DbError> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT id, username, email, locale, created_at FROM users WHERE id = $1",
                &[&id],
            )
            .await?;
        Ok(row.map(|row| User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            locale: row.get("locale"),
            created_at: row.get("created_at"),
        }))
    }

    pub async fn set_user_locale(&self, id: Uuid, locale: &str) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client
            .execute(
                "UPDATE users SET locale = $2 WHERE id = $1",
                &[&id, &locale],
            )
            .await?;
        Ok(())
    }
}
