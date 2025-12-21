use actix_web::{post, get, web, HttpResponse};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder, Set, ActiveModelTrait};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

use crate::models::wallet::{Entity as Wallet, Column as WalletColumn, ActiveModel as WalletActiveModel};
use crate::models::trade::{Entity as Trade, Column as TradeColumn};
use crate::models::stock::{Entity as Stock, Column as StockColumn};  // ← Garde celui-ci
use crate::middleware::AuthUser;

// DTO pour ajouter une transaction
#[derive(Deserialize)]
pub struct AddTransactionRequest {
    pub date: String,           // Format: "2025-12-20"
    pub action: String,         // "gain", "perte", "ajout", "retrait"
    pub symbol: Option<String>, // Optionnel, NULL pour ajout/retrait
    pub amount: f64,
    pub currency: String,       // "CAD", "USD", "EUR"
}

// DTO pour une transaction dans la réponse
#[derive(Serialize)]
pub struct TransactionResponse {
    pub id: i32,
    pub date: String,
    pub action: String,
    pub symbol: Option<String>,
    pub amount: f64,
    pub currency: String,
}

// DTO pour le solde par devise
#[derive(Serialize)]
pub struct BalanceResponse {
    pub currency: String,
    pub total: f64,        // Total du wallet (ajouts + gains - pertes - retraits)
    pub invested: f64,     // Montant investi dans les trades en cours
    pub treasury: f64,     // Trésorerie disponible (total - invested)
}

/// POST /api/wallet/transaction - Ajouter une transaction au wallet
#[post("/transaction")]
pub async fn add_transaction(
    auth_user: AuthUser,
    body: web::Json<AddTransactionRequest>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    // Valider l'action
    let valid_actions = ["gain", "perte", "ajout", "retrait"];
    if !valid_actions.contains(&body.action.as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid action. Must be one of: gain, perte, ajout, retrait"
        }));
    }

    // Valider la devise
    let valid_currencies = ["CAD", "USD", "EUR"];
    if !valid_currencies.contains(&body.currency.as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid currency. Must be one of: CAD, USD, EUR"
        }));
    }

    // Valider le montant
    if body.amount <= 0.0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Amount must be greater than 0"
        }));
    }

    // Convertir f64 en Decimal
    let amount_decimal = match Decimal::from_f64_retain(body.amount) {
        Some(d) => d,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid amount format"
            }));
        }
    };

    // Créer la transaction
    let new_transaction = WalletActiveModel {
        user_id: Set(auth_user.user_id),
        date: Set(body.date.clone()),
        action: Set(body.action.clone()),
        symbol: Set(body.symbol.clone()),
        amount: Set(amount_decimal),
        currency: Set(body.currency.clone()),
        ..Default::default()
    };

    match new_transaction.insert(db.get_ref()).await {
        Ok(transaction) => {
            HttpResponse::Created().json(serde_json::json!({
                "success": true,
                "message": "Transaction added successfully",
                "transaction": {
                    "id": transaction.id,
                    "date": transaction.date,
                    "action": transaction.action,
                    "symbol": transaction.symbol,
                    "amount": decimal_to_f64(transaction.amount),
                    "currency": transaction.currency
                }
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to add transaction: {}", e)
            }))
        }
    }
}

/// GET /api/wallet/history - Récupérer l'historique des transactions
#[get("/history")]
pub async fn get_history(
    auth_user: AuthUser,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    let transactions = Wallet::find()
        .filter(WalletColumn::UserId.eq(auth_user.user_id))
        .order_by_desc(WalletColumn::Date)
        .order_by_desc(WalletColumn::Id)
        .all(db.get_ref())
        .await;

    match transactions {
        Ok(transactions) => {
            let response: Vec<TransactionResponse> = transactions
                .into_iter()
                .map(|t| TransactionResponse {
                    id: t.id,
                    date: t.date,
                    action: t.action,
                    symbol: t.symbol,
                    amount: decimal_to_f64(t.amount),
                    currency: t.currency,
                })
                .collect();

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch history: {}", e)
            }))
        }
    }
}

/// GET /api/wallet/balance - Calculer le solde et la trésorerie par devise
#[get("/balance")]
pub async fn get_balance(
    auth_user: AuthUser,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    // 1. Récupérer toutes les transactions wallet
    let transactions_result = Wallet::find()
        .filter(WalletColumn::UserId.eq(auth_user.user_id))
        .all(db.get_ref())
        .await;

    let transactions = match transactions_result {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch wallet: {}", e)
            }));
        }
    };

    // 2. Récupérer tous les trades (achats et ventes) pour calculer la position nette
    let trades_result = Trade::find()
        .filter(TradeColumn::UserId.eq(auth_user.user_id))
        .all(db.get_ref())
        .await;

    let trades = match trades_result {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch trades: {}", e)
            }));
        }
    };

    // 3. Calculer le solde total par devise (wallet)
    let mut balances: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

    for transaction in transactions {
        let balance = balances.entry(transaction.currency.clone()).or_insert(0.0);
        let amount = decimal_to_f64(transaction.amount);

        match transaction.action.as_str() {
            "gain" | "ajout" => *balance += amount,
            "perte" | "retrait" => *balance -= amount,
            _ => {}
        }
    }

    // 4. Calculer le montant investi par devise
    // On doit joindre avec la table stock pour récupérer la currency de chaque symbole
    use crate::models::stock::{Entity as Stock, Column as StockColumn};

    let mut invested: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

    for trade in trades {
        // Récupérer le symbole du trade
        let symbol = match &trade.symbol {
            Some(s) => s,
            None => continue, // Skip si pas de symbole
        };

        // Trouver le stock correspondant pour récupérer la currency
        let stock = match Stock::find()
            .filter(StockColumn::SymbolAlphavantage.eq(symbol))
            .one(db.get_ref())
            .await
        {
            Ok(Some(s)) => s,
            Ok(None) => {
                // Stock non trouvé, on utilise CAD par défaut
                eprintln!("⚠️  Stock not found for symbol: {}", symbol);
                continue;
            }
            Err(e) => {
                eprintln!("⚠️  Error fetching stock for symbol {}: {}", symbol, e);
                continue;
            }
        };

        // Récupérer la currency du stock (CAD, USD, EUR)
        let currency = stock.currency.unwrap_or_else(|| "CAD".to_string());

        let inv = invested.entry(currency).or_insert(0.0);

        // Calculer le montant investi selon le type de trade
        let quantite = parse_decimal_field(&trade.quantite).unwrap_or(0.0);
        let prix_unitaire = parse_decimal_field(&trade.prix_unitaire).unwrap_or(0.0);
        let montant = quantite * prix_unitaire;

        // Achat: augmente l'investissement, Vente: diminue l'investissement
        match trade.trade_type.as_deref() {
            Some("achat") => *inv += montant,
            Some("vente") => *inv -= montant,
            _ => {} // Type inconnu, on ignore
        }
    }

    // 5. Construire la réponse avec total, invested, treasury
    let mut response: Vec<BalanceResponse> = Vec::new();

    // Récupérer toutes les devises (union des devises du wallet et des trades)
    let mut all_currencies: std::collections::HashSet<String> = balances.keys().cloned().collect();
    all_currencies.extend(invested.keys().cloned());

    for currency in all_currencies {
        let total = *balances.get(&currency).unwrap_or(&0.0);
        let inv = *invested.get(&currency).unwrap_or(&0.0);
        let treasury = total - inv;

        response.push(BalanceResponse {
            currency,
            total,
            invested: inv,
            treasury,
        });
    }

    // Trier par devise
    response.sort_by(|a, b| a.currency.cmp(&b.currency));

    HttpResponse::Ok().json(response)
}

// Fonction helper pour convertir Decimal en f64
fn decimal_to_f64(decimal: Decimal) -> f64 {
    decimal.to_string().parse::<f64>().unwrap_or(0.0)
}

// Fonction helper pour convertir Option<Decimal> en Option<f64>
fn parse_decimal_field(field: &Option<Decimal>) -> Option<f64> {
    field.as_ref().map(|d| decimal_to_f64(*d))
}

pub fn wallet_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/wallet")
            .service(add_transaction)
            .service(get_history)
            .service(get_balance)
    );
}