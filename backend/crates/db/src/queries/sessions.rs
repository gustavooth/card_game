use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{SessionRow, User};
use crate::{Db, DbError};

impl Db {
    pub async fn create_session(
        &self,
        user_id: Uuid,
        token_hash: &[u8],
        expires_at: DateTime<Utc>,
    ) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO sessions (id, user_id, token_hash, expires_at)
                 VALUES ($1, $2, $3, $4)",
                &[&Uuid::new_v4(), &user_id, &token_hash, &expires_at],
            )
            .await?;
        Ok(())
    }

    pub async fn find_session(
        &self,
        token_hash: &[u8],
    ) -> Result<Option<(SessionRow, User)>, DbError> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT s.id AS session_id, s.user_id, s.token_hash, s.created_at AS session_created_at,
                        s.last_seen_at, s.expires_at,
                        u.id AS user_id, u.username, u.email, u.locale, u.created_at AS user_created_at
                 FROM sessions s
                 JOIN users u ON u.id = s.user_id
                 WHERE s.token_hash = $1 AND s.expires_at > now()",
                &[&token_hash],
            )
            .await?;
        Ok(row.map(|row| {
            let session = SessionRow {
                id: row.get("session_id"),
                user_id: row.get("user_id"),
                token_hash: row.get("token_hash"),
                created_at: row.get("session_created_at"),
                last_seen_at: row.get("last_seen_at"),
                expires_at: row.get("expires_at"),
            };
            let user = User {
                id: row.get("user_id"),
                username: row.get("username"),
                email: row.get("email"),
                locale: row.get("locale"),
                created_at: row.get("user_created_at"),
            };
            (session, user)
        }))
    }

    pub async fn delete_session(&self, token_hash: &[u8]) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client
            .execute("DELETE FROM sessions WHERE token_hash = $1", &[&token_hash])
            .await?;
        Ok(())
    }

    pub async fn touch_session(&self, session_id: Uuid) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client
            .execute(
                "UPDATE sessions SET last_seen_at = now() WHERE id = $1",
                &[&session_id],
            )
            .await?;
        Ok(())
    }

    pub async fn prune_expired_sessions(&self) -> Result<u64, DbError> {
        let client = self.pool.get().await?;
        let count = client
            .execute("DELETE FROM sessions WHERE expires_at <= now()", &[])
            .await?;
        Ok(count)
    }
}
