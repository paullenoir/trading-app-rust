use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use serde_json::json;

use crate::services::strategies::strategy_trait::{StrategyCalculator, Recommendation};
use crate::models::indicator::{Entity as Indicator, Column as IndicatorColumn};

pub struct StochasticStrategy;

#[async_trait]
impl StrategyCalculator for StochasticStrategy {
    async fn calculate_batch(
        &self,
        symbols: &[String],
        db: &DatabaseConnection,
    ) -> Result<Vec<Recommendation>, String> {
        println!("🔄 Stochastic Strategy: Processing {} symbols", symbols.len());

        let mut recommendations = Vec::new();

        // Récupérer les derniers indicateurs pour chaque symbole
        for symbol in symbols {
            // Récupérer la dernière ligne pour ce symbole
            let latest_indicator = Indicator::find()
                .filter(IndicatorColumn::Symbol.eq(symbol))
                .order_by_desc(IndicatorColumn::Date)
                .one(db)
                .await
                .map_err(|e| format!("Failed to fetch indicator for {}: {}", symbol, e))?;

            if let Some(indicator) = latest_indicator {
                // Vérifier si Stochastic existe
                if let Some(stoch_str) = &indicator.stochastic14_7_7 {
                    // Parser Stochastic
                    if let Ok(stoch_value) = stoch_str.parse::<f64>() {
                        // Appliquer la logique de stratégie
                        let signal = if stoch_value <= 20.0 {
                            "BUY"
                        } else if stoch_value >= 80.0 {
                            "SELL"
                        } else {
                            "HOLD"
                        };

                        // Créer la recommandation
                        let recommendation = Recommendation {
                            symbol: symbol.clone(),
                            recommendation: json!(signal),
                            metadata: json!({
                                "stochastic14_7_7": stoch_value,
                                "date": indicator.date,
                                "signal_type": signal,
                            }),
                        };

                        recommendations.push(recommendation);
                    }
                }
            }
        }

        println!("✅ Stochastic Strategy: Generated {} recommendations", recommendations.len());
        Ok(recommendations)
    }
}