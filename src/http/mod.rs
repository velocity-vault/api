use actix_web::web::Data;
use actix_web::{HttpServer, App};
use sqlx::MySqlPool;

mod auth_server;
mod auth_user;
mod model;
mod runs;

pub async fn serve(db: MySqlPool) -> anyhow::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db.clone()))
            .configure(auth_user::config)
            .configure(runs::config)
    })
    .bind(("0.0.0.0", 9000))?
    .run()
    .await?;

    Ok(())
}
