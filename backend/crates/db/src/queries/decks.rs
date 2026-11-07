use uuid::Uuid;

use crate::{Db, DbError};

impl Db {
    pub async fn get_deck(&self, user_id: Uuid) -> Result<Option<Vec<String>>, DbError> {
        let client = self.pool.get().await?;
        let row = client
            .query_opt("SELECT cards FROM decks WHERE user_id = $1", &[&user_id])
            .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let value: serde_json::Value = row.get("cards");
        let cards = value
            .as_array()
            .map(|array| {
                array
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(Some(cards))
    }

    pub async fn set_deck(&self, user_id: Uuid, cards: &[String]) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        let value = serde_json::to_value(cards).unwrap_or(serde_json::Value::Array(Vec::new()));
        client
            .execute(
                "INSERT INTO decks (user_id, cards, updated_at) VALUES ($1, $2, now())
                 ON CONFLICT (user_id) DO UPDATE SET cards = EXCLUDED.cards, updated_at = now()",
                &[&user_id, &value],
            )
            .await?;
        Ok(())
    }
}
