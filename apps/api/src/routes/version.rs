use actix_web::{HttpResponse, Responder, get};

#[get("")]
async fn get_version() -> impl Responder {
    HttpResponse::Ok().json(env!("CARGO_PKG_VERSION"))
}

pub fn config_version(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_version);
}
