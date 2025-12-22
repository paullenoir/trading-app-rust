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

TRADES:
  POST /api/trades                          - Créer un trade (achat ou vente) (protégée)
                                              Header: Authorization: Bearer <token>
                                              Body: {
                                                "symbol": "AAPL",
                                                "trade_type": "achat|vente",
                                                "quantite": 10,
                                                "prix_unitaire": 150.50,
                                                "date": "2025-12-20"
                                              }
                                              Response: {
                                                "id": 1,
                                                "user_id": 123,
                                                "symbol": "AAPL",
                                                "trade_type": "achat",
                                                "quantite": 10,
                                                "prix_unitaire": 150.50,
                                                "prix_total": 1505.00,
                                                "date": "2025-12-20"
                                              }
                                              Note: Si type="vente", calcule automatiquement les trades fermés (FIFO)

  GET  /api/trades                          - Voir tous les trades (achats et ventes) (protégée)
                                              Header: Authorization: Bearer <token>
                                              Response: [
                                                {
                                                  "id": 1,
                                                  "user_id": 123,
                                                  "symbol": "AAPL",
                                                  "trade_type": "achat",
                                                  "quantite": 10,
                                                  "prix_unitaire": 150.50,
                                                  "prix_total": 1505.00,
                                                  "date": "2025-12-20"
                                                }
                                              ]

  GET  /api/trades/open                     - Voir les positions ouvertes (calculées FIFO) (protégée)
                                              Header: Authorization: Bearer <token>
                                              Response: [
                                                {
                                                  "symbol": "AAPL",
                                                  "quantite_totale": 10,
                                                  "prix_moyen": 150.50
                                                }
                                              ]

  GET  /api/trades/open-with-recommendations - Voir les positions ouvertes avec recommandations de stratégies (protégée)
                                              Header: Authorization: Bearer <token>
                                              Response: [
                                                {
                                                  "symbol": "AAPL",
                                                  "quantite_totale": 10,
                                                  "prix_moyen": 150.50,
                                                  "strategies": [
                                                    {
                                                      "strategy_id": 1,
                                                      "strategy_name": "RSI",
                                                      "date": "2025-12-20",
                                                      "recommendation": "SELL"
                                                    },
                                                    {
                                                      "strategy_id": 2,
                                                      "strategy_name": "Stochastic",
                                                      "date": "2025-12-20",
                                                      "recommendation": "HOLD"
                                                    }
                                                  ]
                                                }
                                              ]
                                              Note: Combine les positions ouvertes avec les dernières recommandations de stratégies
                                                    pour aider à décider si vendre, garder ou racheter

  GET  /api/trades/closed                   - Voir les trades fermés avec gains/pertes (protégée)
                                              Header: Authorization: Bearer <token>
                                              Response: [
                                                {
                                                  "symbol": "AAPL",
                                                  "date_achat": "2025-12-20",
                                                  "prix_achat": "150.50",
                                                  "date_vente": "2025-12-21",
                                                  "prix_vente": "160.00",
                                                  "pourcentage_gain": 6,
                                                  "gain_dollars": 47.50,
                                                  "temps_jours": 1,
                                                  "trade_achat_id": 1,
                                                  "trade_vente_id": 2
                                                }
                                              ]

========================================
*/

pub mod health;
pub mod stocks;
pub mod admin;
pub mod auth;
pub mod wallet;
pub mod trade;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(health::health_check)
            .configure(stocks::stocks_routes)
            .configure(admin::admin_routes)
            .configure(auth::auth_routes)
            .configure(wallet::wallet_routes)
            .configure(trade::configure)
    );
}