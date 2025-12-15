use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use serde_json::{json, Value};

use crate::services::strategies::strategy_trait::{StrategyCalculator, Recommendation};
use crate::models::indicator::{Entity as Indicator, Column as IndicatorColumn};
use crate::models::historic_data::{Entity as HistoricData, Column as HistoricDataColumn};

/*
========================================
LOGIQUE DE LA STRATÉGIE POINT PIVOT
========================================

1. CONCEPT DE BASE
   - Supports (S1/S2/S3) : Prix où on attend un rebond → Signal BUY
   - Résistances (R1/R2/R3) : Prix où on attend un blocage → Signal SELL
   - Pivot : Point central neutre

2. PONDÉRATION DES PÉRIODES (importance décroissante)
   - Year  : poids = 3 (plus fiable, basé sur 365 jours)
   - Month : poids = 2 (fiable, basé sur 30 jours)
   - Week  : poids = 1 (moins fiable, volatile sur 7 jours)

3. PONDÉRATION DES NIVEAUX (force du support/résistance)
   - S3/R3 : poids = 3 (niveau majeur)
   - S2/R2 : poids = 2 (niveau intermédiaire)
   - S1/R1 : poids = 1 (niveau mineur)

4. DISTANCE "PROCHE"
   - Un prix est "proche" d'un niveau si dans un rayon de 1%
   - Exemple : Si S1 = 100$, proche = [99$ à 101$]

5. CALCUL DU SCORE
   Pour chaque période (year, month, week):
     Pour chaque niveau (S3, S2, S1, R1, R2, R3):
       Si close est proche de ce niveau:
         Score += (poids_période × poids_niveau × direction)

   Direction:
     - Support (S1/S2/S3) → +1 (favorise BUY)
     - Résistance (R1/R2/R3) → -1 (favorise SELL)

6. DÉCISION FINALE
   - Score > 0  → BUY  (plus de supports proches que de résistances)
   - Score < 0  → SELL (plus de résistances proches que de supports)
   - Score = 0  → HOLD (équilibre ou aucun niveau proche)

EXEMPLE:
  Prix = 150.50$
  Year: S1=150.00 (proche) → +3×1×(+1) = +3
  Month: R1=151.00 (proche) → +2×1×(-1) = -2
  Score final = +1 → BUY
========================================
*/

pub struct PointPivotStrategy;

impl PointPivotStrategy {
    /// Vérifie si le prix est "proche" d'un niveau (dans un rayon de 1%)
    fn is_close_to_level(&self, price: f64, level: f64) -> bool {
        let threshold = level * 0.01; // 1% du niveau
        (price - level).abs() <= threshold
    }

    /// Calcule le score pour une période donnée (year/month/week)
    fn calculate_period_score(
        &self,
        close: f64,
        period_pivots: &Value,
        period_weight: i32,
    ) -> i32 {
        let mut score = 0;

        // Extraire les niveaux de cette période
        let s3 = period_pivots["s3"].as_f64();
        let s2 = period_pivots["s2"].as_f64();
        let s1 = period_pivots["s1"].as_f64();
        let r1 = period_pivots["r1"].as_f64();
        let r2 = period_pivots["r2"].as_f64();
        let r3 = period_pivots["r3"].as_f64();

        // Vérifier chaque niveau de support (direction = +1 pour BUY)
        if let Some(s3_val) = s3 {
            if self.is_close_to_level(close, s3_val) {
                score += period_weight * 3 * 1; // poids_période × poids_niveau × direction
            }
        }
        if let Some(s2_val) = s2 {
            if self.is_close_to_level(close, s2_val) {
                score += period_weight * 2 * 1;
            }
        }
        if let Some(s1_val) = s1 {
            if self.is_close_to_level(close, s1_val) {
                score += period_weight * 1 * 1;
            }
        }

        // Vérifier chaque niveau de résistance (direction = -1 pour SELL)
        if let Some(r1_val) = r1 {
            if self.is_close_to_level(close, r1_val) {
                score += period_weight * 1 * (-1); // poids_période × poids_niveau × direction
            }
        }
        if let Some(r2_val) = r2 {
            if self.is_close_to_level(close, r2_val) {
                score += period_weight * 2 * (-1);
            }
        }
        if let Some(r3_val) = r3 {
            if self.is_close_to_level(close, r3_val) {
                score += period_weight * 3 * (-1);
            }
        }

        score
    }
}

#[async_trait]
impl StrategyCalculator for PointPivotStrategy {
    async fn calculate_batch(
        &self,
        symbols: &[String],
        db: &DatabaseConnection,
    ) -> Result<Vec<Recommendation>, String> {
        println!("🔄 Point Pivot Strategy: Processing {} symbols", symbols.len());

        let mut recommendations = Vec::new();

        for symbol in symbols {
            // Récupérer le dernier indicateur pour ce symbole
            let latest_indicator = Indicator::find()
                .filter(IndicatorColumn::Symbol.eq(symbol))
                .order_by_desc(IndicatorColumn::Date)
                .one(db)
                .await
                .map_err(|e| format!("Failed to fetch indicator for {}: {}", symbol, e))?;

            if let Some(indicator) = latest_indicator {
                let date = &indicator.date;

                // Récupérer le close du même jour
                let historic = HistoricData::find()
                    .filter(HistoricDataColumn::Symbol.eq(symbol))
                    .filter(HistoricDataColumn::Date.eq(date))
                    .one(db)
                    .await
                    .map_err(|e| format!("Failed to fetch historic data for {}: {}", symbol, e))?;

                if let Some(historic_data) = historic {
                    if let Some(close_str) = &historic_data.close {
                        if let Ok(close) = close_str.parse::<f64>() {
                            // Récupérer les point pivots (JSON)
                            if let Some(point_pivot) = &indicator.point_pivot {
                                let mut total_score = 0;

                                // Calculer score pour year (poids = 3)
                                if let Some(year_pivots) = point_pivot.get("year") {
                                    if !year_pivots.is_null() && year_pivots.as_object().is_some() {
                                        total_score += self.calculate_period_score(close, year_pivots, 3);
                                    }
                                }

                                // Calculer score pour month (poids = 2)
                                if let Some(month_pivots) = point_pivot.get("month") {
                                    if !month_pivots.is_null() && month_pivots.as_object().is_some() {
                                        total_score += self.calculate_period_score(close, month_pivots, 2);
                                    }
                                }

                                // Calculer score pour week (poids = 1)
                                if let Some(week_pivots) = point_pivot.get("week") {
                                    if !week_pivots.is_null() && week_pivots.as_object().is_some() {
                                        total_score += self.calculate_period_score(close, week_pivots, 1);
                                    }
                                }

                                // Décision finale basée sur le score
                                let signal = if total_score > 0 {
                                    "BUY"
                                } else if total_score < 0 {
                                    "SELL"
                                } else {
                                    "HOLD"
                                };

                                // Créer la recommandation
                                let recommendation = Recommendation {
                                    symbol: symbol.clone(),
                                    recommendation: json!(signal),
                                    metadata: json!({
                                        "close": close,
                                        "total_score": total_score,
                                        "signal_type": signal,
                                        "date": date,
                                        "point_pivot": point_pivot,
                                    }),
                                };

                                recommendations.push(recommendation);
                            }
                        }
                    }
                }
            }
        }

        println!("✅ Point Pivot Strategy: Generated {} recommendations", recommendations.len());
        Ok(recommendations)
    }
}