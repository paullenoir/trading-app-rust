/*
services/
├─ strategy_service.rs
│  ├─ execute_default_strategies()     ← ADMIN, 5 stratégies hardcodées
│  └─ execute_custom_strategy()        ← USER, parse JSON DSL (futur)
│
└─ strategies/
   ├─ strategy_trait.rs                ← Interface commune
   ├─ defaults/                        ← Stratégies ADMIN hardcodées
   │  ├─ mod.rs
   │  ├─ min_max_last_year.rs
   │  ├─ rsi.rs
   │  ├─ stochastic.rs
   │  ├─ ema.rs
   │  └─ point_pivot.rs
   │
   └─ custom/                           ← Interpréteur JSON DSL (futur)
      ├─ mod.rs
      └─ dsl_executor.rs                ← Parse strategy_config
*/
use sea_orm::{DatabaseConnection, Set, ActiveModelTrait, EntityTrait, QueryFilter, ColumnTrait, IntoActiveModel};
use chrono::Local;

use crate::services::strategies::{
    strategy_trait::{StrategyCalculator, Recommendation},
    defaults::{
        min_max_last_year::MinMaxLastYear,
        rsi::RSIStrategy,
        stochastic::StochasticStrategy,
        ema::EMAStrategy,
        point_pivot::PointPivotStrategy,
    },
};
use crate::services::indicator_service::IndicatorService;
use crate::models::{
    strategy_result::{self, Entity as StrategyResult},
    stock::Entity as Stock,
};

pub struct StrategyService;

impl StrategyService {
    //constructeur
    //-> Self : retourne une instance de strategyservice
    pub fn new() -> Self {
        Self //le type lui-même (StrategyService)
    }

    // FLOW 1: ADMIN - Stratégies par défaut hardcodées
    pub async fn execute_default_strategies(
        &self,
        db: &DatabaseConnection,
    ) -> Result<Vec<Recommendation>, String> {
        println!("🚀 Starting strategy execution");

        // 1. Récupérer tous les symboles
        let stocks = Stock::find()
            .all(db)
            .await
            .map_err(|e| format!("Failed to fetch stocks: {}", e))?;

        let symbols: Vec<String> = stocks
            .into_iter()
            .filter_map(|s| s.symbol_alphavantage)
            .collect();

        //test 1 symbol
        //let symbols: Vec<String> = vec!["AAPL".to_string()];

        println!("📊 Found {} symbols", symbols.len());

        // 2. Calculer les indicateurs (RSI, EMA, Stochastic, point_pivot)
        let indicator_service = IndicatorService::new();
        indicator_service.calculate_all_indicators(symbols.clone(), db).await?;

        println!("✅ Indicators calculated");

        // 3. Exécuter les stratégies
        let mut all_results = Vec::new();

        // ============================================================================
        // STRATÉGIE 1 : MinMaxLastYear (strategy_id = 1)
        // ============================================================================
        println!("📊 Executing MinMaxLastYear strategy...");
        let min_max_calc = MinMaxLastYear;
        let min_max_recs = min_max_calc.calculate_batch(&symbols, db).await?;
        println!("✅ Calculated {} recommendations for MinMaxLastYear", min_max_recs.len());

        for rec in min_max_recs {
            save_result(1, &rec.symbol, &rec, db).await?;
            all_results.push(rec);
        }

        // ============================================================================
        // STRATÉGIE 2 : RSI (strategy_id = 2)
        // ============================================================================
        println!("📊 Executing RSI strategy...");
        let rsi_calc = RSIStrategy;
        let rsi_recs = rsi_calc.calculate_batch(&symbols, db).await?;
        println!("✅ Calculated {} recommendations for RSI", rsi_recs.len());

        for rec in rsi_recs {
            save_result(2, &rec.symbol, &rec, db).await?;
            all_results.push(rec);
        }

        // ============================================================================
        // STRATÉGIE 3 : Stochastic (strategy_id = 3)
        // ============================================================================
        println!("📊 Executing Stochastic strategy...");
        let stoch_calc = StochasticStrategy;
        let stoch_recs = stoch_calc.calculate_batch(&symbols, db).await?;
        println!("✅ Calculated {} recommendations for Stochastic", stoch_recs.len());

        for rec in stoch_recs {
            save_result(3, &rec.symbol, &rec, db).await?;
            all_results.push(rec);
        }

        // ============================================================================
        // STRATÉGIE 4 : EMA (strategy_id = 4)
        // ============================================================================
        println!("📊 Executing EMA strategy...");
        let ema_calc = EMAStrategy;
        let ema_recs = ema_calc.calculate_batch(&symbols, db).await?;
        println!("✅ Calculated {} recommendations for EMA", ema_recs.len());

        for rec in ema_recs {
            save_result(4, &rec.symbol, &rec, db).await?;
            all_results.push(rec);
        }

        // ============================================================================
        // STRATÉGIE 5 : Point Pivot (strategy_id = 5)
        // ============================================================================
        println!("📊 Executing Point Pivot strategy...");
        let pivot_calc = PointPivotStrategy;
        let pivot_recs = pivot_calc.calculate_batch(&symbols, db).await?;
        println!("✅ Calculated {} recommendations for Point Pivot", pivot_recs.len());

        for rec in pivot_recs {
            save_result(5, &rec.symbol, &rec, db).await?;
            all_results.push(rec);
        }

        println!("✅ Strategy execution completed: {} total recommendations", all_results.len());

        Ok(all_results)
    }

    // FLOW 2: USER - Stratégies custom via JSON DSL (futur)
    pub async fn execute_custom_strategy(
        &self,
        _strategy_id: i32,
        _symbols: Vec<String>,
        _db: &DatabaseConnection,
    ) -> Result<Vec<Recommendation>, String> {
        // TODO: Lire strategy_config, parser JSON DSL, exécuter dynamiquement
        todo!("Custom strategies not implemented yet")
    }
}

// Fonction helper pour sauvegarder un résultat dans strategy_results_test
async fn save_result(
    strategy_id: i32,
    symbol: &str,
    rec: &Recommendation,
    db: &DatabaseConnection,
) -> Result<(), String> {
    let today = Local::now().naive_local().date().format("%Y-%m-%d").to_string();

    // 1. Chercher si un enregistrement existe déjà
    let existing = StrategyResult::find()
        .filter(strategy_result::Column::StrategyId.eq(strategy_id))
        .filter(strategy_result::Column::Symbol.eq(symbol))
        .one(db)
        .await
        .map_err(|e| format!("Failed to query existing result: {}", e))?;

    match existing {
        // 2a. Si existe → UPDATE
        Some(existing_model) => {
            let mut active_model: strategy_result::ActiveModel = existing_model.into_active_model();
            active_model.recommendation = Set(Some(rec.recommendation.clone()));
            active_model.metadata = Set(Some(rec.metadata.clone()));

            active_model.update(db)
                .await
                .map_err(|e| format!("Failed to update result: {}", e))?;
        }

        // 2b. Si n'existe pas → INSERT
        None => {
            let new_model = strategy_result::ActiveModel {
                strategy_id: Set(strategy_id),
                symbol: Set(Some(symbol.to_string())),
                date: Set(Some(today)),
                recommendation: Set(Some(rec.recommendation.clone())),
                metadata: Set(Some(rec.metadata.clone())),
                ..Default::default()
            };

            new_model.insert(db)
                .await
                .map_err(|e| format!("Failed to insert result: {}", e))?;
        }
    }

    Ok(())
}