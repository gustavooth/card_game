use uuid::Uuid;

use crate::models::PlayerStats;
use crate::{Db, DbError};

impl Db {
    pub async fn player_stats(&self, user_id: Uuid) -> Result<PlayerStats, DbError> {
        let client = self.pool.get().await?;
        let row = client
            .query_one(
                "SELECT
                    COUNT(*) AS games,
                    COUNT(*) FILTER (WHERE winner = $1) AS wins,
                    COUNT(*) FILTER (WHERE winner IS NOT NULL AND winner <> $1) AS losses,
                    COUNT(*) FILTER (WHERE winner IS NULL) AS draws,
                    COALESCE(SUM(CASE WHEN player_blue = $1 THEN blue_crowns ELSE red_crowns END), 0)::BIGINT AS crowns_for,
                    COALESCE(SUM(CASE WHEN player_blue = $1 THEN red_crowns ELSE blue_crowns END), 0)::BIGINT AS crowns_against
                 FROM matches
                 WHERE player_blue = $1 OR player_red = $1",
                &[&user_id],
            )
            .await?;
        Ok(PlayerStats {
            games: row.get("games"),
            wins: row.get("wins"),
            losses: row.get("losses"),
            draws: row.get("draws"),
            crowns_for: row.get("crowns_for"),
            crowns_against: row.get("crowns_against"),
        })
    }
}
