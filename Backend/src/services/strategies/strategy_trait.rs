use sea_orm::DatabaseConnection;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use async_trait::async_trait;

#[derive(Debug, Serialize, Deserialize)]
pub struct Recommendation {
    pub symbol: String,
    pub recommendation: Value,  // JSON flexible : "BUY" ou ["BUY", "SELL", "BUY"]
    pub metadata: Value,         // JSON flexible pour les métriques spécifiques
}

//trait = Interface
#[async_trait]
pub trait StrategyCalculator {
    // Méthode pour 1 symbole (simple) - OPTIONNELLE avec implémentation par défaut
    async fn calculate(
        &self,
        _symbol: &str,
        _config: &Value,
        _db: &DatabaseConnection,
    ) -> Result<Recommendation, String> {
        Err("Single symbol calculation not implemented for this strategy".to_string())
    }

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