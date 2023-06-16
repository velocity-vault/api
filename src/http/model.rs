use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize)]
pub enum RunKind {
    NUB,
    PRO,
}

#[derive(Serialize, FromRow)]
pub struct MapRun {
    player_id: u64,
    player_name: Option<String>,
    ticks: u32,
    teleports: u32,
    created_at: DateTime<Utc>,
}
