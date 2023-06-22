use actix_web::get;
use actix_web::error::Result;
use actix_web::web::{ServiceConfig, Json, Data};
use sqlx::mysql::MySqlPool;
use super::model::Mode;

pub fn config(conf: &mut ServiceConfig) {
    conf.service(get_modes);
}

#[get("/get_modes")]
async fn get_modes(db: Data<MySqlPool>) -> Result<Json<Vec<Mode>>> {
    let result: Vec<Mode> = sqlx::query_as(r#"
        SELECT m.name, m.short_name
        FROM modes m
        ORDER BY m.mode_id
    "#)
    .fetch_all(db.get_ref()).await
    .map_err(|_| actix_web::error::ErrorInternalServerError(""))?;

    Ok(Json(result))
}
