/*
========================================
ROUTES DISPONIBLES
========================================

HEALTH:
  GET  /api/health                          - Vérifier que l'API fonctionne

STOCKS:
  GET  /api/stocks                          - Récupérer tous les stocks
  GET  /api/stocks/with-strategies          - Récupérer les stocks avec leurs stratégies (dernière date)

ADMIN:
  POST /api/admin/strategies/calculate      - Calculer les indicateurs et stratégies pour tous les symboles
                                              (RSI, Stochastic, EMA, Point Pivot, MinMaxLastYear)

AUTH:
  POST /api/auth/register                   - Créer un compte utilisateur
                                              Body: {"username": "...", "password": "..."}
                                              Response: {"token": "...", "user_id": 123, "username": "..."}

  POST /api/auth/login                      - Se connecter
                                              Body: {"username": "...", "password": "..."}
                                              Response: {"token": "...", "user_id": 123, "username": "..."}

  GET  /api/auth/me                         - Vérifier son token JWT (route protégée)
                                              Header: Authorization: Bearer <token>
                                              Response: {"user_id": 123, "username": "..."}

  POST /api/auth/change-password            - Changer son mot de passe (route protégée)
                                              Header: Authorization: Bearer <token>
                                              Body: {"current_password": "...", "new_password": "..."}
                                              Response: {"success": true, "message": "Password changed successfully"}

TODO - WALLET (à venir):
  POST /api/wallet/deposit                  - Ajouter de l'argent au wallet
  POST /api/wallet/withdraw                 - Retirer de l'argent du wallet
  GET  /api/wallet/history                  - Voir l'historique du wallet
  GET  /api/wallet/balance                  - Voir les soldes (CAD, USD, EUR)

TODO - TRADES (à venir):
  POST /api/trades                          - Acheter une action (créer trade ouvert)
  PUT  /api/trades/:id/sell                 - Vendre une action (fermer trade)
  GET  /api/trades                          - Voir tous les trades ouverts
  GET  /api/trades/closed                   - Voir l'historique des trades fermés

========================================
*/

pub mod health;
pub mod stocks;
pub mod admin;
pub mod auth;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(health::health_check)
            .configure(stocks::stocks_routes)
            .configure(admin::admin_routes)
            .configure(auth::auth_routes)
    );
}