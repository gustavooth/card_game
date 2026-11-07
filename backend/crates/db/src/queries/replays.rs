use uuid::Uuid;

use crate::models::ReplayMeta;
use crate::{Db, DbError};

impl Db {
    pub async fn insert_replay(
        &self,
        user_id: Uuid,
        match_id: Uuid,
        data: &[u8],
    ) -> Result<Uuid, DbError> {
        let mut client = self.pool.get().await?;
        let id = Uuid::new_v4();
        let transaction = client.transaction().await?;
        transaction
            .execute(
                "INSERT INTO replays (id, match_id, user_id, data) VALUES ($1, $2, $3, $4)",
                &[&id, &match_id, &user_id, &data],
            )
            .await?;
        transaction
            .execute(
                "DELETE FROM replays
                 WHERE user_id = $1
                   AND id NOT IN (
                       SELECT id FROM replays WHERE user_id = $1 ORDER BY created_at DESC LIMIT 10
                   )",
                &[&user_id],
            )
            .await?;
        transaction.commit().await?;
        Ok(id)
    }

    pub async fn list_replays(&self, user_id: Uuid) -> Result<Vec<ReplayMeta>, DbError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT id, match_id, created_at FROM replays
                 WHERE user_id = $1 ORDER BY created_at DESC LIMIT 10",
                &[&user_id],
            )
            .await?;
        Ok(rows
            .into_iter()
            .map(|row| ReplayMeta {
                id: row.get("id"),
                match_id: row.get("match_id"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    pub async fn get_replay_data(
        &self,
        user_id: Uuid,
        replay_id: Uuid,
    ) -> Result<Option<Vec<u8>>, DbError> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt(
                "SELECT data FROM replays WHERE id = $1 AND user_id = $2",
                &[&replay_id, &user_id],
            )
            .await?;
        Ok(row.map(|row| row.get("data")))
    }
}
