use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use serde_json::json;

use crate::services::strategies::strategy_trait::{StrategyCalculator, Recommendation};
use crate::models::indicator::{Entity as Indicator, Column as IndicatorColumn};
use crate::models::historic_data::{Entity as HistoricData, Column as HistoricDataColumn};

pub struct EMAStrategy;

#[async_trait]
impl StrategyCalculator for EMAStrategy {
    async fn calculate_batch(
        &self,
        symbols: &[String],
        db: &DatabaseConnection,
    ) -> Result<Vec<Recommendation>, String> {
        println!("🔄 EMA Strategy: Processing {} symbols", symbols.len());

        let mut recommendations = Vec::new();

        // Récupérer les derniers indicateurs pour chaque symbole
        for symbol in symbols {
            // Récupérer la dernière ligne d'indicateurs pour ce symbole
            let latest_indicator = Indicator::find()
                .filter(IndicatorColumn::Symbol.eq(symbol))
                .order_by_desc(IndicatorColumn::Date)
                .one(db)
                .await
                .map_err(|e| format!("Failed to fetch indicator for {}: {}", symbol, e))?;

            if let Some(indicator) = latest_indicator {
                let date = &indicator.date;

                // Récupérer le close du même jour depuis historicdata
                let historic = HistoricData::find()
                    .filter(HistoricDataColumn::Symbol.eq(symbol))
                    .filter(HistoricDataColumn::Date.eq(date))
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to fetch historic data for {}: {}", symbol, e))?;

                if let Some(historic_data) = historic {
                    if let Some(close_str) = &historic_data.close {
                        if let Ok(close) = close_str.parse::<f64>() {
                            // Parser les 3 EMAs
                            let ema20 = indicator.ema20.as_ref().and_then(|s| s.parse::<f64>().ok());
                            let ema50 = indicator.ema50.as_ref().and_then(|s| s.parse::<f64>().ok());
                            let ema200 = indicator.ema200.as_ref().and_then(|s| s.parse::<f64>().ok());

                            // Calculer les 3 signaux
                            let mut signals = Vec::new();

                            // Signal 1 : Close vs EMA20
                            if let Some(ema20_val) = ema20 {
                                signals.push(if close > ema20_val { "BUY" } else { "SELL" });
                            } else {
                                signals.push("N/A");
                            }

                            // Signal 2 : Close vs EMA50
                            if let Some(ema50_val) = ema50 {
                                signals.push(if close > ema50_val { "BUY" } else { "SELL" });
                            } else {
                                signals.push("N/A");
                            }

                            // Signal 3 : Close vs EMA200
                            if let Some(ema200_val) = ema200 {
                                signals.push(if close > ema200_val { "BUY" } else { "SELL" });
                            } else {
                                signals.push("N/A");
                            }

                            // Créer la recommandation avec Vec<String>
                            let recommendation = Recommendation {
                                symbol: symbol.clone(),
                                recommendation: json!(signals), // ["BUY", "SELL", "BUY"]
                                metadata: json!({
                                    "close": close,
                                    "ema20": ema20,
                                    "ema50": ema50,
                                    "ema200": ema200,
                                    "date": date,
                                    "signals": signals,
                                }),
                            };

                            recommendations.push(recommendation);
                        }
                    }
                }
            }
        }

        println!("✅ EMA Strategy: Generated {} recommendations", recommendations.len());
        Ok(recommendations)
    }
}