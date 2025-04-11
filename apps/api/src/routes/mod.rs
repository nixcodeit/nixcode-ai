use actix_web::{Scope, web};

mod version;

pub fn configure_routes() -> Scope {
    web::scope("/api").configure(|cfg| {
        cfg.service(web::scope("version").configure(version::config_version));
    })
}
