use sea_orm::DatabaseConnection;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use async_trait::async_trait;

#[derive(Debug, Serialize, Deserialize)]
pub struct Recommendation {
    pub symbol: String,
    pub recommendation: String,  // "BUY", "SELL", "HOLD"
    pub metadata: Value,  // JSON flexible pour les métriques spécifiques
}

//trait = Interface
#[async_trait]
pub trait StrategyCalculator {
    // Méthode pour 1 symbole (simple)
    async fn calculate(
        &self,
        symbol: &str,
        config: &Value,
        db: &DatabaseConnection,
    ) -> Result<Recommendation, String>;

    // Méthode batch pour plusieurs symboles (optimisée)
    async fn calculate_batch(
        &self,
        symbols: &[String],
        db: &DatabaseConnection,
    ) -> Result<Vec<Recommendation>, String> {
        // Implémentation par défaut : boucle sur calculate()
        // Les stratégies peuvent override pour optimiser
        let mut results = Vec::new();
        for symbol in symbols {
            let rec = self.calculate(symbol, &Value::Null, db).await?;
            results.push(rec);
        }
        Ok(results)
    }
}