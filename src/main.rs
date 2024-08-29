use actix_web::{error, get, web, App, HttpServer, Responder, Result};
use actix_files::Files;
use log::info;
use tera::{Context, Tera};

use crate::database::Database;

mod config;
mod database;
mod legacy;
mod types;

#[get("/")]
async fn index(db: web::Data<Database>, tmpl: web::Data<Tera>) -> Result<impl Responder, actix_web::Error> {
    let mut ctx = Context::new();
    let mirrors = db.get_all_mirrors().await.map_err(|e| error::ErrorInternalServerError(e))?;
    ctx.insert("mirrors", &mirrors);
    let html = tmpl.render("index.html", &ctx).map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(web::Html::new(html))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let conf = config::try_load()?;

    let db = Database::try_init(&conf.database_url, conf.database_pool_size).await?;
    info!("Connected to database at {}", conf.database_url);

    let tera = Tera::new(&(conf.files_dir.clone() + "/templates/*"))?;

    Ok(HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(tera.clone()))
            .service(Files::new("/static", conf.files_dir.clone() + "/static"))
            .service(index)
            .configure(legacy::config)
    })
    .bind(conf.bind_addr)?
    .run()
    .await?)
}
