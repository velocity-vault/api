use actix_web::error::Result;
use actix_web::{get, Responder, HttpResponse};
use actix_web::web::{ServiceConfig, Data, Query, Json};
use serde::Deserialize;
use sqlx::mysql::MySqlPool;
use super::model::{Map, Player};

pub fn config(conf: &mut ServiceConfig) {
    conf.service(search_players)
        .service(search_maps);
}

#[derive(Deserialize)]
struct SearchPlayers { 
    query: String,
}

#[get("/search_players")]
async fn search_players(query: Query<SearchPlayers>, db: Data<MySqlPool>) -> Result<Json<Vec<Player>>> {
    // We're using MATCH IN BOOLEAN MODE, which lets you use some operators in the string query. 
    // I don't think they let the user do anything nefarious, but they might get unexpected results,
    // so I prefer nuking them.
    let search_str = query.query
        .split(['+', '-', '@', '>', '<', '(', ')', '~', '*', '"', '%', ' '])
        .filter(|w| w.len() >= 2)
        .map(|s| [s, "*"].concat())
        .collect::<Vec<String>>()
        .join(" ");
    if search_str.is_empty() {
        return Err(actix_web::error::ErrorBadRequest("insufficient search query"));
    }

    let result: Vec<Player> = sqlx::query_as(r#"
        SELECT p.player_id AS id, p.name
        FROM players p
        WHERE MATCH(p.name) AGAINST (? IN BOOLEAN MODE)
        ORDER BY p.search_relevance DESC
        LIMIT 20
    "#)
    .bind(&search_str)
    .fetch_all(db.get_ref()).await
    .map_err(|_| actix_web::error::ErrorInternalServerError(""))?;

    Ok(Json(result))
}

#[derive(Deserialize)]
struct SearchMaps { 
    query: String,
    mode: String,
}

#[get("/search_maps")]
async fn search_maps(query: Query<SearchMaps>, db: Data<MySqlPool>) -> Result<Json<Vec<Map>>> {
    // We're using MATCH IN BOOLEAN MODE, which lets you use some operators in the string query. 
    // I don't think they let the user do anything nefarious, but they might get unexpected results,
    // so I prefer nuking them.
    // ... We're also nuking the underscore, since it should be treated as whitespace.
    let search_str = query.query
        .split(['+', '-', '@', '<', '>', '(', ')', '~', '*', '"', '%', ' ', '_'])
        .filter(|w| w.len() >= 2)
        .map(|s| [s, "*"].concat())
        .collect::<Vec<String>>()
        .join(" ");
    if search_str.is_empty() {
        return Err(actix_web::error::ErrorBadRequest("insufficient search query"));
    }

    let result: Vec<Map> = sqlx::query_as(r#"
        SELECT m.name, m.created_at,
            CASE WHEN c.num IS NULL 
                THEN JSON_ARRAY()
                ELSE JSON_ARRAYAGG(DISTINCT JSON_OBJECT(
                    'course', c.num,
                    'nub_tier', f.nub_tier,
                    'pro_tier', f.pro_tier
                )) 
            END AS courses,
            CASE WHEN ma.mapper_id IS NULL
                THEN JSON_ARRAY()
                ELSE JSON_ARRAYAGG(DISTINCT JSON_OBJECT(
                    'id', p.player_id, 
                    'name', p.name
                )) 
            END AS mappers
        FROM maps m
        INNER JOIN (
            SELECT MATCH(m.search_tags) AGAINST (? IN BOOLEAN MODE) AS score, m.map_id
            FROM maps m
            ORDER BY score DESC
            LIMIT 20
        ) s ON s.map_id = m.map_id 
        LEFT JOIN mappers ma ON ma.map_id = m.map_id
        LEFT JOIN players p ON p.player_id = ma.player_id
        INNER JOIN courses c ON c.map_id = m.map_id
        INNER JOIN modes m2 ON m2.short_name = ?
        INNER JOIN filters f ON f.course_id = c.course_id AND f.mode_id = m2.mode_id
        WHERE s.score > 0
        GROUP BY m.map_id
        ORDER BY s.score DESC
    "#)
    .bind(&search_str)
    .bind(&query.mode)
    .fetch_all(db.get_ref()).await
    .map_err(|_| actix_web::error::ErrorInternalServerError(""))?;

    Ok(Json(result))
}
