//pour la réponse structurée
use serde::Serialize;

// 1 objet StockWithStrategies
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
    pub date: Option<chrono::NaiveDate>,
    pub recommendation: Option<String>,
}