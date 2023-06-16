use actix_web::dev::Payload;
use actix_web::web::Data;
use actix_web::{FromRequest, HttpRequest, Result};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use sqlx::FromRow;
use std::future::Future;
use std::pin::Pin;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Server {
    id: u32,
}

impl Server {
    pub fn id(self: &Self) -> u32 {
        self.id
    }
}

impl FromRequest for Server {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let Some(token) = req.headers().get("X-Server-Token") else {
            return Box::pin(async move { Err(actix_web::error::ErrorBadRequest("X-Server-Token header missing")) });
        };
        let Ok(token) = token.to_str() else {
            return Box::pin(async move { Err(actix_web::error::ErrorBadRequest("X-Server-Token must be ASCII")) });
        };
        let token = token.to_owned();
        let db = req.app_data::<Data<MySqlPool>>().unwrap().get_ref().to_owned();
        Box::pin(async move {
            let server: Server = sqlx::query_as(r#"
                SELECT s.server_id AS id
                FROM servers s
                WHERE s.token = ?
                LIMIT 1
            "#)
            .bind(token)
            .fetch_optional(&db).await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?
            .ok_or(actix_web::error::ErrorUnauthorized("X-Server-Token is invalid"))?;
            Ok(server)
        })
    }
}
