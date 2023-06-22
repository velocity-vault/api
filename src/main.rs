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
        .connect(&db_url).await
        .context("could not connect to the database")?;

    //sqlx::migrate!().run(&db).await?;
    http::serve(db).await?;

    Ok(())
}
