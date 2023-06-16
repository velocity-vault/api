use anyhow::Context;
use sqlx::mysql::MySqlPoolOptions;
use std::env;

mod http;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db = MySqlPoolOptions::new()
        .max_connections(50)
        .connect(&db_url)
        .await
        .context("could not connect to the database")?;

    //sqlx::migrate!().run(&db).await?;
    http::serve(db).await?;

    Ok(())
}

/*
//
// Auth
//

#[derive(Serialize, Deserialize)]
struct Claims {
    userid: i64,
    permissions: Vec<String>,
    exp: i64,
}

impl Claims {
    pub fn new(userid: i64, permissions: Vec<String>) -> Self {
        Self {
            userid,
            permissions,
            exp: (Utc::now() + Duration::hours(2)).timestamp(),
        }
    }
}

fn encode_jwt(claims: Claims) -> jsonwebtoken::errors::Result<String> {
    let encoding_key = EncodingKey::from_secret("abcdef".as_bytes());
    jsonwebtoken::encode(&Header::default(), &claims, &encoding_key)
}

fn decode_jwt(tokenx: &str) -> jsonwebtoken::errors::Result<Claims> {
    let decoding_key = DecodingKey::from_secret("abcdef".as_bytes());
    jsonwebtoken::decode::<Claims>(tokenx, &decoding_key, &Validation::default())
        .map(|data| data.claims)
}

async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    println!("validator called!");
    let result = decode_jwt(credentials.token());
    match result {
        Ok(claims) => {
            req.attach(claims.permissions);
            Ok(req)
        }
        Err(_) => Err((actix_web::error::ErrorForbidden("oops"), req))
    }
}

#[get("/token")]
async fn token() -> actix_web::Result<impl Responder> {
    let mut permissions = Vec::new();
    permissions.push("VIEW_BANS".to_owned());

    let claims = Claims::new(123456789, permissions);
    let jwt = encode_jwt(claims)
        .map_err(|_| actix_web::error::ErrorInternalServerError("oops"))?;

    Ok(jwt)
}

#[get("/protected")]
#[has_permissions("VIEW_BANS")]
async fn protected() -> impl Responder {
    "You accessed a protected route!!!"
}

#[get("/unprotected")]
async fn unprotected() -> impl Responder {
    "You accessed an unprotected route!!!"
}
*/
