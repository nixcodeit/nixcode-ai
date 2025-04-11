mod routes;

use crate::routes::configure_routes;
use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .service(configure_routes())
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
