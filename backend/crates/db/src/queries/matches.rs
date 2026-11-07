use uuid::Uuid;

use crate::models::{NewMatch, RecentMatch};
use crate::{Db, DbError};

impl Db {
    pub async fn insert_match(&self, record: &NewMatch) -> Result<(), DbError> {
        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO matches (
                    id, player_blue, player_red, winner, result_blue, result_red,
                    duration_ms, blue_crowns, red_crowns, blue_damage, red_damage,
                    end_reason, started_at, ended_at
                 ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
                &[
                    &record.id,
                    &record.player_blue,
                    &record.player_red,
                    &record.winner,
                    &record.result_blue,
                    &record.result_red,
                    &record.duration_ms,
                    &record.blue_crowns,
                    &record.red_crowns,
                    &record.blue_damage,
                    &record.red_damage,
                    &record.end_reason,
                    &record.started_at,
                    &record.ended_at,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn recent_matches(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<RecentMatch>, DbError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT m.id, m.ended_at, m.duration_ms,
                        COALESCE(u.username, '?') AS opponent,
                        CASE
                            WHEN m.winner IS NULL THEN 'draw'
                            WHEN m.winner = $1 THEN 'win'
                            ELSE 'loss'
                        END AS result,
                        CASE WHEN m.player_blue = $1 THEN m.blue_crowns ELSE m.red_crowns END AS crowns,
                        CASE WHEN m.player_blue = $1 THEN m.red_crowns ELSE m.blue_crowns END AS opponent_crowns
                 FROM matches m
                 LEFT JOIN users u ON u.id = CASE
                     WHEN m.player_blue = $1 THEN m.player_red ELSE m.player_blue
                 END
                 WHERE m.player_blue = $1 OR m.player_red = $1
                 ORDER BY m.ended_at DESC
                 LIMIT $2",
                &[&user_id, &limit],
            )
            .await?;
        Ok(rows
            .into_iter()
            .map(|row| RecentMatch {
                id: row.get("id"),
                ended_at: row.get("ended_at"),
                duration_ms: row.get("duration_ms"),
                opponent: row.get("opponent"),
                result: row.get("result"),
                crowns: row.get("crowns"),
                opponent_crowns: row.get("opponent_crowns"),
            })
            .collect())
    }
}
