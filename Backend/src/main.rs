/*
============================================================================
VISION DU PROJET - APPLICATION DE TRADING EN 3 VERSIONS
============================================================================

VERSION 1 - FONDATIONS & SUIVI MANUEL (ACTUELLE)
-------------------------------------------------
Plateforme de suivi de trades avec authentification JWT sécurisée et gestion
de wallet multi-devises (CAD/USD/EUR). Le système calcule automatiquement les
positions ouvertes en FIFO, les trades clôturés avec gains/pertes, et fournit
des recommandations quotidiennes via 5 stratégies par défaut (RSI, Stochastic,
EMA, Point Pivot, MinMaxLastYear) pour l'ensemble des symboles. Architecture
Rust/Actix-Web/SeaORM optimisée pour traiter 2000+ symboles en batch.

VERSION 2 - STRATÉGIES PERSONNALISÉES VIA CHATGPT + MCP
--------------------------------------------------------
Permet aux utilisateurs de créer leurs propres stratégies de trading sans coder,
via une conversation avec ChatGPT connecté à un serveur MCP. L'utilisateur décrit
sa stratégie en langage naturel, ChatGPT génère un DSL JSON qui encode la logique,
et le backend l'exécute de manière sécurisée. Limites: max 10 stratégies par user,
15 symboles par stratégie (150 symboles total). Inclut un mode backtesting pour
valider les stratégies sur données historiques avant activation.

VERSION 3 - TRADING AUTOMATIQUE AVEC INTERACTIVE BROKERS
---------------------------------------------------------
Automatisation complète du trading avec exécution temps réel via Interactive Brokers.
Chaque utilisateur peut créer un "agent trader IA" qui analyse continuellement les
marchés selon ses stratégies personnalisées et exécute automatiquement les ordres.
Gestion des risques critique: stop-loss/take-profit automatiques, limites de position,
perte max journalière, circuit breaker. Sécurité renforcée: 2FA obligatoire,
chiffrement credentials IB, alertes email/SMS pour chaque ordre, monitoring 24/7
avec bouton d'arrêt d'urgence. Modes: paper trading (simulation), dry-run (analyse
seule), et live trading (exécution réelle).

============================================================================
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