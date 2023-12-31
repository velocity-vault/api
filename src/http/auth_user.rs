use actix_web::web::{self, Data, Json, ServiceConfig};
use actix_web::{get, FromRequest, Result, HttpResponse, HttpRequest};
use chrono::{Duration, Utc};
use futures::TryFutureExt;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{env, future::Future, pin::Pin};
use steam_openid::SteamOpenId;
use super::model::AuthUserResponse;

pub fn config(conf: &mut ServiceConfig) {
    conf.app_data(Data::new(LocalData::new()))
        .service(steam_auth)
        .service(steam_auth_verify)
        .service(get_protected);
}

#[get("/steam_auth")]
async fn steam_auth(data: Data<LocalData>) -> HttpResponse {
    let location = data.steam_openid.get_redirect_url();
    HttpResponse::PermanentRedirect()
        .append_header(("Location", location))
        .append_header(("Cache-Control", "no-store"))
        .finish()
}

#[get("/steam_auth_verify")]
async fn steam_auth_verify(req: HttpRequest, data: Data<LocalData>) -> Result<HttpResponse> {
    let steamid64 = data.steam_openid.verify(req.query_string()).await
        .map_err(|_| actix_web::error::ErrorUnauthorized("Verification failed"))?;
    const STEAMID64_BASE: u64 = 76561197960265728;
    if steamid64 <= STEAMID64_BASE {
        return Err(actix_web::error::ErrorInternalServerError("Steam oopsie, please send help"));
    }

    let user_id = steamid64 - STEAMID64_BASE;
    let permissions = Vec::new();
    let claims = Claims::new(user_id, permissions, Duration::hours(2));
    let token = jsonwebtoken::encode(&Header::default(), &claims, &data.encoding_key)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to encode token"))?;

    let result = AuthUserResponse {
        player_id: user_id,
        token,
    };
    Ok(HttpResponse::Ok().json(result))
}

#[get("/protected")]
async fn get_protected(user: User) -> Result<Json<Vec<Permission>>> {
    user_guard(user.has_permission(Permission::ViewBans))?;
    Ok(web::Json(user.permissions))
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum Permission {
    ViewBans,
    ViewMaps,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    user_id: u64,
    permissions: Vec<Permission>,
    exp: i64,
}

impl Claims {
    fn new(user_id: u64, permissions: Vec<Permission>, valid_for: Duration) -> Self {
        Self {
            user_id,
            permissions,
            exp: (Utc::now() + valid_for).timestamp(),
        }
    }
}

pub struct User {
    id: u64,
    permissions: Vec<Permission>,
}

impl User {
    fn new(id: u64, permissions: Vec<Permission>) -> Self {
        Self {
            id,
            permissions,
        }
    }
    pub fn id(self: &Self) -> u64 {
        self.id
    }
    pub fn has_permission(self: &Self, p: Permission) -> bool {
        self.permissions.contains(&p)
    }
}

impl FromRequest for User {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let Some(token) = req.headers().get("X-User-Token") else {
            return Box::pin(async move { Err(actix_web::error::ErrorBadRequest("X-User-Token header missing")) });
        };
        let Ok(token) = token.to_str() else {
            return Box::pin(async move { Err(actix_web::error::ErrorBadRequest("X-User-Token must be ASCII")) });
        };
        let data = req.app_data::<Data<LocalData>>().unwrap();
        let Ok(decoded) = jsonwebtoken::decode::<Claims>(&token, &data.decoding_key, &Validation::default()) else {
            return Box::pin(async move { Err(actix_web::error::ErrorUnauthorized("X-User-Token is invalid")) });
        };
        Box::pin(async move { Ok(User::new(decoded.claims.user_id, decoded.claims.permissions)) })
    }
}

pub fn user_guard(condition: bool) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(actix_web::error::ErrorForbidden("forbidden"))
    }
}

struct LocalData {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    steam_openid: SteamOpenId,
}

impl LocalData {
    fn new() -> Self {
        let auth_token_secret = env::var("AUTH_TOKEN_SECRET").expect("AUTH_TOKEN_SECRET not set");
        assert!(auth_token_secret.len() >= 32);
        Self {
            encoding_key: EncodingKey::from_secret(auth_token_secret.as_ref()),
            decoding_key: DecodingKey::from_secret(auth_token_secret.as_ref()),
            steam_openid: SteamOpenId::new("http://localhost:5000", "#/steam_auth").unwrap(),
        }
    }
}
