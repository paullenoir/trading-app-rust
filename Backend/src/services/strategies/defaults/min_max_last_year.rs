use crate::services::strategies::strategy_trait::{StrategyCalculator, Recommendation};
use sea_orm::DatabaseConnection;
use serde_json::{Value, json};
use chrono::{Local, Duration};
use async_trait::async_trait;
use sqlx::Row;

// ========== CONSTANTES ==========
const CALCULATION_PERIOD_DAYS: i64 = 365;
const BUY_THRESHOLD: f64 = 20.0;   // En dessous de 20% = BUY
const SELL_THRESHOLD: f64 = 80.0;  // Au-dessus de 80% = SELL
// ================================

pub struct MinMaxLastYear;

#[async_trait]
impl StrategyCalculator for MinMaxLastYear {
    async fn calculate(
        &self,
        _symbol: &str,
        _config: &Value,
        _db: &DatabaseConnection,
    ) -> Result<Recommendation, String> {
        // Cette méthode n'est plus utilisée, on utilise calculate_batch
        Err("Use calculate_batch for optimized performance".to_string())
    }

    async fn calculate_batch(
        &self,
        _symbols: &[String],
        db: &DatabaseConnection,
    ) -> Result<Vec<Recommendation>, String> {
        // Calculer la date de cutoff
        let one_year_ago = Local::now().naive_local().date() - Duration::days(CALCULATION_PERIOD_DAYS);
        let cutoff_date = one_year_ago.format("%Y-%m-%d").to_string();

        // Appeler la stored procedure PostgreSQL
        let pool = db.get_postgres_connection_pool();
        let rows = sqlx::query("SELECT * FROM get_min_max_prices_last_year($1)")
            .bind(&cutoff_date)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("SQL stored procedure error: {}", e))?;

        // Transformer les résultats en Recommendations
        let mut results = Vec::new();

        for row in rows {
            let symbol: String = row.try_get("symbol")
                .map_err(|e| format!("Failed to get symbol: {}", e))?;

            let min_price: f64 = row.try_get("min_price")
                .map_err(|e| format!("Failed to get min_price for {}: {}", symbol, e))?;

            let max_price: f64 = row.try_get("max_price")
                .map_err(|e| format!("Failed to get max_price for {}: {}", symbol, e))?;

            let current_price: Option<f64> = row.try_get("current_price").ok();

            // Validation des données
            let current_price = match current_price {
                Some(price) if price > 0.0 => price,
                _ => {
                    println!("⚠️ Skipping {} - no current price", symbol);
                    continue;
                }
            };

            if max_price == min_price {
                println!("⚠️ Skipping {} - no price variation (min=max)", symbol);
                continue;
            }

            // Calculer le pourcentage (côté Rust)
            let percentage = ((current_price - min_price) / (max_price - min_price)) * 100.0;

            // Déterminer la recommandation avec les constantes
            let recommendation = if percentage <= BUY_THRESHOLD {
                "BUY"
            } else if percentage >= SELL_THRESHOLD {
                "SELL"
            } else {
                "HOLD"
            };

            results.push(Recommendation {
                symbol: symbol.clone(),
                recommendation: json!(recommendation),
                metadata: json!({
                    "percentage": format!("{:.2}", percentage),
                    "min_price": format!("{:.2}", min_price),
                    "max_price": format!("{:.2}", max_price),
                    "current_price": format!("{:.2}", current_price),
                    "calculation_period_days": CALCULATION_PERIOD_DAYS,
                    "buy_threshold": BUY_THRESHOLD,
                    "sell_threshold": SELL_THRESHOLD
                }),
            });
        }

        Ok(results)
    }
}