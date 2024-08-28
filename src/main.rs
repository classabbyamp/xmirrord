use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use log::info;

use crate::database::Database;

mod config;
mod database;
mod legacy;
mod types;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("hewoo????")
}

#[get("/metrics/prometheus.json")]
async fn metrics() -> impl Responder {
    HttpResponse::Ok().body("{}")
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let conf = config::try_load()?;

    let db = Database::try_init(&conf.database_url, conf.database_pool_size).await?;
    info!("Connected to database at {}", conf.database_url);

    Ok(HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .service(index)
            .service(metrics)
            .configure(legacy::config)
    })
    .bind(conf.bind_addr)?
    .run()
    .await?)
}
