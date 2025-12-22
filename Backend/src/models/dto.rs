use serde::{Deserialize, Serialize};
use validator::Validate;
use rust_decimal::Decimal;

// ============================================
// DTOs pour Stocks et Stratégies
// ============================================

#[derive(Debug, Serialize)]
pub struct StockWithStrategies {
    pub stock: StockInfo,
    pub strategies: Vec<StrategyWithResult>,
}

#[derive(Debug, Serialize)]
pub struct StockInfo {
    pub company_name: String,
    pub symbol_alphavantage: Option<String>,
    pub currency: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StrategyWithResult {
    pub strategy_id: i32,
    pub strategy_name: Option<String>,
    pub date: Option<String>,
    pub recommendation: Option<String>,
}

// ============================================
// DTOs pour Trades
// ============================================

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTradeRequest {
    #[validate(length(min = 1))]
    pub symbol: String,

    #[validate(custom(function = "validate_trade_type"))]
    pub trade_type: String,

    #[validate(custom(function = "validate_positive_decimal"))]
    pub quantite: Decimal,

    #[validate(custom(function = "validate_positive_decimal"))]
    pub prix_unitaire: Decimal,

    pub date: String,
}

#[derive(Debug, Serialize)]
pub struct TradeResponse {
    pub id: i32,
    pub user_id: i32,
    pub symbol: String,
    pub trade_type: String,
    pub quantite: Decimal,
    pub prix_unitaire: Decimal,
    pub prix_total: Decimal,
    pub date: String,
}

#[derive(Debug, Serialize)]
pub struct OpenPositionResponse {
    pub symbol: String,
    pub quantite_totale: Decimal,
    pub prix_moyen: Decimal,
}

#[derive(Debug, Serialize)]
pub struct OpenPositionWithRecommendationsResponse {
    pub symbol: String,
    pub quantite_totale: Decimal,
    pub prix_moyen: Decimal,
    pub strategies: Vec<StrategyWithResult>,
}

#[derive(Debug, Serialize)]
pub struct ClosedTradeResponse {
    pub symbol: String,
    pub date_achat: String,
    pub prix_achat: String,
    pub date_vente: String,
    pub prix_vente: String,
    pub pourcentage_gain: i32,
    pub gain_dollars: Decimal,
    pub temps_jours: i32,
    pub trade_achat_id: i32,
    pub trade_vente_id: i32,
}

fn validate_trade_type(value: &str) -> Result<(), validator::ValidationError> {
    if value == "achat" || value == "vente" {
        Ok(())
    } else {
        Err(validator::ValidationError::new("invalid_trade_type"))
    }
}

fn validate_positive_decimal(value: &Decimal) -> Result<(), validator::ValidationError> {
    if value > &Decimal::ZERO {
        Ok(())
    } else {
        Err(validator::ValidationError::new("must_be_positive"))
    }
}