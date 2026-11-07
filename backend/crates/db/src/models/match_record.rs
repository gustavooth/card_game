use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct RecentMatch {
    pub id: Uuid,
    pub ended_at: DateTime<Utc>,
    pub duration_ms: i32,
    pub opponent: String,
    pub result: String,
    pub crowns: i16,
    pub opponent_crowns: i16,
}

#[derive(Debug, Clone)]
pub struct NewMatch {
    pub id: Uuid,
    pub player_blue: Uuid,
    pub player_red: Uuid,
    pub winner: Option<Uuid>,
    pub result_blue: String,
    pub result_red: String,
    pub duration_ms: i32,
    pub blue_crowns: i16,
    pub red_crowns: i16,
    pub blue_damage: f32,
    pub red_damage: f32,
    pub end_reason: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchRecord {
    pub id: Uuid,
    pub player_blue: Uuid,
    pub player_red: Uuid,
    pub winner: Option<Uuid>,
    pub duration_ms: i32,
    pub ended_at: DateTime<Utc>,
}
