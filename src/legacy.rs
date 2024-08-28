use actix_web::{error, get, web, HttpResponse, Responder, Result};
use csv::WriterBuilder;
use fred::error::RedisError;
use serde::Serialize;

use crate::{database::Database, types::{Mirror, Region, Tier, Protocol}};

#[derive(Debug, Clone, Serialize)]
struct LegacyMirror {
    region: Region,
    baseurl: String,
    location: String,
    tier: Tier,
    enabled: bool,
}

impl From<&Mirror> for LegacyMirror {
    fn from(value: &Mirror) -> Self {
        Self {
            region: value.region.clone(),
            baseurl: format!("{}://{}/", value.protocols.get(0).unwrap_or(&Protocol::Http), value.baseurl),
            location: value.location.clone(),
            tier: value.tier.clone(),
            enabled: value.enabled,
        }
    }
}

async fn get_all_mirrors_legacy(db: web::Data<Database>) -> std::result::Result<Vec<LegacyMirror>, RedisError> {
    match db.get_all_mirrors().await {
        Ok(mirrors) => Ok(mirrors.into_iter().filter(|m| m.enabled).map(|m| LegacyMirror::from(&m)).collect()),
        Err(e) => Err(e),
    }
}

#[get("/v0/mirrors.json")]
async fn v0_mirrors(db: web::Data<Database>) -> Result<impl Responder> {
    match get_all_mirrors_legacy(db).await {
        Ok(mirrors) => Ok(web::Json(mirrors)),
        Err(e) => Err(error::ErrorInternalServerError(e))
    }
}

#[get("/raw/mirrors.lst")]
async fn raw_mirrors(db: web::Data<Database>) -> impl Responder {
    match get_all_mirrors_legacy(db).await {
        Ok(mirrors) => {
            let mut wtr = WriterBuilder::new()
                .has_headers(false)
                .delimiter(b'\t')
                .from_writer(vec![]);
            // TODO
            for m in mirrors {
                wtr.serialize(m).expect("blah");
            }
            let data = String::from_utf8(wtr.into_inner().expect("blep")).expect("bruh");
            Ok(HttpResponse::Ok().body(data))
        }
        Err(e) => Err(error::ErrorInternalServerError(e))
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(v0_mirrors).service(raw_mirrors);
}
