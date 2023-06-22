use actix_web::{get};
use actix_web::error::Result;
use actix_web::web::{ServiceConfig, Json, Data, Query};
use serde::Deserialize;
use sqlx::mysql::MySqlPool;
use super::model::Map;

pub fn config(conf: &mut ServiceConfig) {
    conf.service(get_maps);
}

#[derive(Deserialize)]
struct GetMaps { 
    mode: String,
}

#[get("/maps")]
async fn get_maps(query: Query<GetMaps>, db: Data<MySqlPool>) -> Result<Json<Vec<Map>>> {
    let result: Vec<Map> = sqlx::query_as(r#"
        SELECT m.name, m.created_at,
            CASE WHEN c.num IS NULL 
                THEN JSON_ARRAY()  
                ELSE JSON_ARRAYAGG(DISTINCT JSON_OBJECT(
                    'course', c.num,
                    'nubTier', f.nub_tier,
                    'proTier', f.pro_tier
                ) ORDER BY c.num ASC) 
            END AS courses,
            CASE WHEN ma.mapper_id IS NULL
                THEN JSON_ARRAY()
                ELSE JSON_ARRAYAGG(DISTINCT JSON_OBJECT(
                    'id', p.player_id, 
                    'name', p.name
                )) 
            END AS mappers
        FROM maps m
        LEFT JOIN mappers ma ON ma.map_id = m.map_id
        LEFT JOIN players p ON p.player_id = ma.player_id
        INNER JOIN courses c ON c.map_id = m.map_id
        INNER JOIN modes m2 ON m2.short_name = ?
        INNER JOIN filters f ON f.course_id = c.course_id AND f.mode_id = m2.mode_id 
        GROUP BY m.map_id 
    "#)
    .bind(&query.mode)
    .fetch_all(db.get_ref())
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError(""))?;

    Ok(Json(result))
}
