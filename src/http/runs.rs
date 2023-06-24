use actix_web::error::Result;
use actix_web::get;
use actix_web::web::{ServiceConfig, Json, Query, Data};
use serde::Deserialize;
use sqlx::MySqlPool;
use super::model::{MapRun, Run, RunKind};

pub fn config(conf: &mut ServiceConfig) {
    conf.service(get_maptop)
        .service(get_course_pb_history);
}

#[derive(Deserialize)]
pub struct GetMapTop { 
    map: String,
    course: u32,
    mode: String,
    kind: RunKind,
}

#[get("/get_maptop")]
async fn get_maptop(query: Query<GetMapTop>, db: Data<MySqlPool>) -> Result<Json<Vec<MapRun>>> {
    let (index, teleports) = match query.kind {
        RunKind::NUB => ("idx_runs__filterid_playerid_ticks_createdat", "1"),
        RunKind::PRO => ("idx_runs__filterid_tps_playerid_ticks_createdat", "teleports = 0"),
    };
    let result: Vec<MapRun> = sqlx::query_as(&format!(r#"
        SELECT r.player_id, p.name AS player_name, t.ticks, r.teleports, r.created_at
        FROM runs r
        USE INDEX({index})
        INNER JOIN players p ON p.player_id = r.player_id 
        INNER JOIN (
            SELECT r.player_id, f.filter_id, MIN(r.ticks) AS ticks
            FROM runs r
            USE INDEX({index})
            INNER JOIN filters f ON f.filter_id = r.filter_id 
            INNER JOIN courses c ON c.course_id = f.course_id 
            INNER JOIN maps m ON m.map_id = c.map_id 
            INNER JOIN modes m2 ON m2.mode_id = f.mode_id 
            WHERE m.name = ? AND c.num = ? AND m2.short_name = ? AND {teleports}
            GROUP BY r.player_id 
            ORDER BY ticks ASC
            LIMIT 50
        ) t ON t.player_id = r.player_id AND t.filter_id = r.filter_id AND t.ticks = r.ticks
        WHERE {teleports}
        GROUP BY r.player_id
        ORDER BY ticks ASC
    "#))
    .bind(&query.map)
    .bind(query.course)
    .bind(&query.mode)
    .fetch_all(db.get_ref()).await
    .map_err(|_| actix_web::error::ErrorInternalServerError(""))?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct GetCoursePbHistory {
    player_id: u64,
    map: String,
    course: u32,
    mode: String,
    kind: RunKind,
}

#[get("/get_course_pb_history")]
async fn get_course_pb_history(query: Query<GetCoursePbHistory>, db: Data<MySqlPool>) -> Result<Json<Vec<Run>>> {
    let (index, and_r_teleports, and_r2_teleports) = match query.kind {
        RunKind::NUB => ("idx_runs__filterid_playerid_ticks_createdat", "", ""),
        RunKind::PRO => ("idx_runs__filterid_tps_playerid_ticks_createdat", "AND r.teleports = 0", "AND r2.teleports = 0"),
    };
    let result: Vec<Run> = sqlx::query_as(&format!(r#"
        SELECT r.ticks, r.teleports, r.created_at 
        FROM runs r 
          USE INDEX({index})
        INNER JOIN filters f ON f.filter_id = r.filter_id 
        INNER JOIN courses c ON c.course_id = f.course_id 
        INNER JOIN maps m ON m.map_id = c.map_id 
        INNER JOIN modes m2 ON m2.mode_id = f.mode_id 
        LEFT OUTER JOIN runs r2 
          USE INDEX({index}) 
            ON r2.player_id = r.player_id 
            AND r2.filter_id = r.filter_id
            {and_r2_teleports}
            AND r2.ticks <= r.ticks 
            AND r2.created_at < r.created_at 
        WHERE r2.run_id IS NULL
            AND r.player_id = ?
            AND m.name = ?
            AND c.num = ?
            AND m2.short_name = ?
            {and_r_teleports}
        ORDER BY r.ticks ASC
        LIMIT 50
    "#))
    .bind(query.player_id)
    .bind(&query.map)
    .bind(query.course)
    .bind(&query.mode)
    .fetch_all(db.get_ref()).await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    Ok(Json(result))
}