use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PlayerStats {
    pub games: i64,
    pub wins: i64,
    pub losses: i64,
    pub draws: i64,
    pub crowns_for: i64,
    pub crowns_against: i64,
}
