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

WALLET:
  POST /api/wallet/transaction              - Ajouter une transaction au wallet (protégée)
                                              Header: Authorization: Bearer <token>
                                              Body: {
                                                "date": "2025-12-20",
                                                "action": "ajout|retrait|gain|perte",
                                                "symbol": "AAPL" (optionnel, null pour ajout/retrait),
                                                "amount": 100.50,
                                                "currency": "CAD|USD|EUR"
                                              }
                                              Response: {"success": true, "message": "Transaction added successfully", "transaction": {...}}

  GET  /api/wallet/history                  - Voir l'historique des transactions (protégée)
                                              Header: Authorization: Bearer <token>
                                              Response: [
                                                {
                                                  "id": 1,
                                                  "date": "2025-12-20",
                                                  "action": "ajout",
                                                  "symbol": null,
                                                  "amount": 1000.0,
                                                  "currency": "CAD"
                                                }
                                              ]

  GET  /api/wallet/balance                  - Voir les soldes et trésorerie par devise (protégée)
                                              Header: Authorization: Bearer <token>
                                              Response: [
                                                {
                                                  "currency": "CAD",
                                                  "total": 2500.50,      // Total wallet (ajouts + gains - retraits - pertes)
                                                  "invested": 1800.00,   // Montant investi dans les trades en cours
                                                  "treasury": 700.50     // Trésorerie disponible (total - invested)
                                                }
                                              ]

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
pub mod wallet;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(health::health_check)
            .configure(stocks::stocks_routes)
            .configure(admin::admin_routes)
            .configure(auth::auth_routes)
            .configure(wallet::wallet_routes)
    );
}