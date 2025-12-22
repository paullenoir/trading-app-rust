/*
voici un zip provenant de mon github pour mon projet app de trading.
il y a 2 volet back end et front end.
pour le moment je te demande que analyser la partie backend end rust.
le but de cette version 1 du backend est d'obtenir pour un user les trades en cours
avec les recommandation de strategie, toutes les info sur son wallet, les trades_fermes donc les trades cloture
et les recommandation des startegie par defaut pour tous les symbol.
dans la version 2 je ferais en sorte que les users puisse utliser leur chatGPT connecte a
un serveur MCP que jaurai fait afin de créé un startegie dont la logique sera stocke en BD
et le user pourra la tester sur 15 symbols et ensuite l'utiliser pour ses 15 symbols.
le user pourra avoir un maximum de 10 strategies pour un maximum de 150 symbols.
dans la version 3 je ferai en sorte que le user puisse cree une IA et faire du trade automatique
avec interactive brocker
*/

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