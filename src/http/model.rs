use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, FromRow};
use sqlx::mysql::MySqlRow;

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

#[derive(Serialize, Deserialize, FromRow)]
pub struct Player {
    id: u64,
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Course {
    course: u32,
    nub_tier: Option<u32>,
    pro_tier: Option<u32>,
}

#[derive(Serialize)]
pub struct Map {
    name: String,
    courses: Vec<Course>,
    mappers: Vec<Player>,
    created_at: DateTime<Utc>,
}

impl<'c> FromRow<'c, MySqlRow> for Map {
    fn from_row(row: &'c MySqlRow) -> sqlx::Result<Self> {
        let courses: Vec<Course> = serde_json::from_str(row.try_get("courses")?)
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        let mappers: Vec<Player> = serde_json::from_str(row.try_get("mappers")?)
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        Ok(Map {
            name: row.try_get("name")?,
            courses,
            mappers,
            created_at: row.try_get("created_at")?,
        })
    }
}

#[derive(Serialize, FromRow)]
pub struct Mode {
    name: String,
    short_name: String,
}
