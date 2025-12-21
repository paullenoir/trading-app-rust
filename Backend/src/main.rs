mod models;
mod routes;
mod db;
mod services;
mod utils;
mod middleware;
use actix_web::{App, HttpServer, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    println!("🔌 Connecting to database...");
    let db = db::establish_connection()
        .await
        .expect("Failed to connect to database");
    println!("✅ Database connected!");

    println!("🚀 Starting server on http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(routes::configure_routes)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}