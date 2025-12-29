// ============================================================================
// MODELS - MODULE PRINCIPAL
// ============================================================================
// 
// Description:
//   Point d'entrée pour tous les modèles de données.
//   Chaque modèle correspond à une table PostgreSQL avec SeaORM.
//
// Liste des modules:
//   - health : Health check API
//   - stock : Symboles boursiers (AAPL, TSLA, etc.)
//   - strategy : Définitions des stratégies de trading
//   - strategy_result : Résultats des stratégies calculées
//   - historic_data : Données historiques OHLCV
//   - indicator : Indicateurs techniques (RSI, EMA, etc.)
//   - dto : Data Transfer Objects pour les réponses API
//   - users : Utilisateurs (auth classique + OAuth Google)
//   - password_reset_tokens : Tokens de reset password (expire 1h)
//   - email_verification_tokens : Tokens de vérification email (expire 24h)
//   - wallet : Transactions wallet (ajout/retrait/gain/perte)
//   - trade : Trades (achats/ventes)
//   - trades_fermes : Historique trades fermés (FIFO)
//   - abonnement : Plans d'abonnement (Free, Pro, etc.)
//
// Points d'attention:
//   - Tous les modèles utilisent SeaORM (pas de SQL brut)
//   - Les tables ont le suffixe "_rust" pour coexister avec Python
//   - Les relations entre tables sont définies dans chaque modèle
//
// ============================================================================

pub mod health;
pub mod stock;
pub mod strategy;
pub mod strategy_result;
pub mod historic_data;
pub mod indicator;
pub mod dto;
pub mod users;
pub mod password_reset_tokens;
pub mod email_verification_tokens;
pub mod wallet;
pub mod trade;
pub mod trades_fermes;
pub mod abonnement;