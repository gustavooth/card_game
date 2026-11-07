use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct ReplayMeta {
    pub id: Uuid,
    pub match_id: Uuid,
    pub created_at: DateTime<Utc>,
}
