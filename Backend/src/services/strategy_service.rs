/*
services/
├─ strategy_service.rs
│  ├─ execute_default_strategies()     ← ADMIN, 4 stratégies hardcodées
│  └─ execute_custom_strategy()        ← USER, parse JSON DSL (futur)
│
└─ strategies/
   ├─ strategy_trait.rs                ← Interface commune
   ├─ defaults/                        ← Stratégies ADMIN hardcodées
   │  ├─ mod.rs
   │  ├─ min_max_last_year.rs
   │  ├─ rsi.rs
   │  ├─ stochastic.rs
   │  └─ ema_triple_crossover.rs
   │
   └─ custom/                           ← Interpréteur JSON DSL (futur)
      ├─ mod.rs
      └─ dsl_executor.rs                ← Parse strategy_config
*/
use sea_orm::{DatabaseConnection, Set, ActiveModelTrait, EntityTrait, QueryFilter, ColumnTrait, IntoActiveModel};
use chrono::Local;

use crate::services::strategies::{
    strategy_trait::{StrategyCalculator, Recommendation},
    defaults::min_max_last_year::MinMaxLastYear,
};
use crate::models::strategy_result::{self, Entity as StrategyResult};

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
        symbols: Vec<String>,
        db: &DatabaseConnection,
    ) -> Result<Vec<Recommendation>, String> {
        let mut all_results = Vec::new();

        // MinMaxLastYear avec query batch optimisée
        let calculator = MinMaxLastYear;
        let recommendations = calculator.calculate_batch(&symbols, db).await?;

        println!("✅ Calculated {} recommendations for MinMaxLastYear", recommendations.len());

        // Sauvegarder tous les résultats
        for rec in recommendations {
            save_result(1, &rec.symbol, &rec, db).await?;
            all_results.push(rec);
        }

        // TODO: Ajouter les autres stratégies (RSI, Stochastic, EMA)
        // let rsi_calc = RSIStrategy;
        // let rsi_recs = rsi_calc.calculate_batch(&symbols, db).await?;
        // for rec in rsi_recs { save_result(3, &rec.symbol, &rec, db).await?; }

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

// Fonction helper pour sauvegarder un résultat dans strategy_results
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
        //.filter(strategy_result::Column::Date.eq(&today))
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