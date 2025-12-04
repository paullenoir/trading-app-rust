mod models;
mod routes;

use actix_web::{App, HttpServer};
use routes::health::health_check;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .service(health_check)
    })
        .bind("127.0.0.1:8080")?
        .run().await
}