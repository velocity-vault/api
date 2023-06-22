use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{HttpServer, App};
use sqlx::MySqlPool;

mod auth_server;
mod auth_user;
mod model;
mod runs;
mod maps;
mod modes;
mod search;

pub async fn serve(db: MySqlPool) -> anyhow::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(Data::new(db.clone()))
            .configure(auth_user::config)
            .configure(runs::config)
            .configure(maps::config)
            .configure(modes::config)
            .configure(search::config)
    })
    .bind(("0.0.0.0", 9000))?
    .run().await?;

    Ok(())
}
