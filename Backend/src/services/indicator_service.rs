use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder, Set, ActiveModelTrait, QuerySelect, TransactionTrait};
use sea_orm::sea_query::Expr;
use chrono::{NaiveDate, Duration};
use polars::prelude::*;
use std::collections::HashSet;

use crate::models::{
    indicator::{Entity as Indicator, Column as IndicatorColumn, ActiveModel as IndicatorActiveModel},
    historic_data::{self, Entity as HistoricData},
};
use crate::services::indicators::rsi::RSICalculator;
use crate::services::indicators::stochastic::StochasticCalculator;
use crate::services::indicators::ema::EMACalculator;
use crate::services::indicators::point_pivot::PointPivotCalculator;

pub struct IndicatorService;

impl IndicatorService {
    pub fn new() -> Self {
        Self
    }

    pub async fn calculate_all_indicators(
        &self,
        symbols: Vec<String>,
        db: &DatabaseConnection,
    ) -> Result<String, String> {
        println!("📊 Starting indicator calculation for {} symbols", symbols.len());

        // 1. Identifier les symboles existants vs nouveaux
        let symbols_in_indicators = self.get_existing_symbols(db).await?;

        let existing_symbols: Vec<String> = symbols.iter()
            .filter(|s| symbols_in_indicators.contains(*s))
            .cloned()
            .collect();

        let new_symbols: Vec<String> = symbols.iter()
            .filter(|s| !symbols_in_indicators.contains(*s))
            .cloned()
            .collect();

        println!("📊 Existing symbols: {}, New symbols: {}", existing_symbols.len(), new_symbols.len());

        let mut total_inserted = 0;

        // 2. FLUX A : Symboles existants (incrémental)
        if !existing_symbols.is_empty() {
            let count = self.process_existing_symbols(&existing_symbols, db).await?;
            total_inserted += count;
        }

        // 3. FLUX B : Nouveaux symboles (full)
        if !new_symbols.is_empty() {
            let count = self.process_new_symbols(&new_symbols, db).await?;
            total_inserted += count;
        }

        Ok(format!("Calculated and saved {} indicator records", total_inserted))
    }

    /// Récupère la liste des symboles présents dans la table indicators (indicator_test en DEV)
    async fn get_existing_symbols(&self, db: &DatabaseConnection) -> Result<HashSet<String>, String> {
        let symbols = Indicator::find()
            .select_only()
            .column(IndicatorColumn::Symbol)
            .distinct()
            .into_tuple::<String>()
            .all(db)
            .await
            .map_err(|e| format!("Failed to get existing symbols: {}", e))?;

        Ok(symbols.into_iter().collect())
    }

    /// FLUX A : Traite les symboles existants (incrémental)
    async fn process_existing_symbols(&self, symbols: &[String], db: &DatabaseConnection) -> Result<usize, String> {
        println!("🔄 FLUX A: Processing existing symbols (incremental)");

        // 1. Récupérer la dernière date globale
        let last_date_result = Indicator::find()
            .select_only()
            .column_as(Expr::col(IndicatorColumn::Date).max(), "max_date")
            .into_tuple::<Option<String>>()
            .one(db)
            .await
            .map_err(|e| format!("Failed to get last date: {}", e))?;

        //Some(Some("2024-12-13")) → Some("2024-12-13") ✅
        //Some(None) → None (MAX() a retourné NULL)
        //None → None (aucune ligne retournée)
        let last_date = match last_date_result.flatten() {
            Some(date) => date,
            None => return Ok(0),
        };

        println!("📅 Last date in indicators: {}", last_date);

        // 2. Calculer cutoff (365 jours avant)
        let last_date_parsed = NaiveDate::parse_from_str(&last_date, "%Y-%m-%d")
            .map_err(|e| format!("Date parse error: {}", e))?;
        let cutoff = last_date_parsed - Duration::days(365);
        let cutoff_str = cutoff.format("%Y-%m-%d").to_string();

        println!("📅 Fetching historicdata from {} onwards", cutoff_str);

        // 3. Fetch historicdata (365 jours pour les symboles existants uniquement)
        let df_full = self.fetch_historicdata_after(&cutoff_str, symbols, db).await?;
        println!("📊 df_full: {} rows", df_full.height());

        if df_full.height() == 0 {
            println!("⚠️  No historical data found");
            return Ok(0);
        }

        // 4. Filtrer seulement les nouvelles dates (> last_date)
        let df_new_dates = df_full.clone().lazy()
            .filter(col("date").gt(lit(last_date.as_str())))
            .collect()
            .map_err(|e| format!("Failed to filter new dates: {}", e))?;

        println!("📋 df_new_dates: {} rows (new trading days)", df_new_dates.height());

        if df_new_dates.height() == 0 {
            println!("✅ No new dates to process");
            return Ok(0);
        }

        // 5. Calculer RSI + Stochastic + EMA + Point Pivot
        let rsi_calculator = RSICalculator::new(25);
        let stoch_calculator = StochasticCalculator::new(14, 7, 7);
        let ema_calculator = EMACalculator::new(vec![20, 50, 200]);
        let pivot_calculator = PointPivotCalculator::new();

        let df_rsi = rsi_calculator.calculate(df_new_dates.clone(), &df_full)
            .map_err(|e| format!("RSI calculation error: {}", e))?;

        let df_stoch = stoch_calculator.calculate(df_new_dates.clone(), &df_full)
            .map_err(|e| format!("Stochastic calculation error: {}", e))?;

        let df_ema = ema_calculator.calculate(df_new_dates.clone(), &df_full)
            .map_err(|e| format!("EMA calculation error: {}", e))?;

        let df_pivot = pivot_calculator.calculate(df_new_dates.clone(), &df_full)
            .map_err(|e| format!("Point Pivot calculation error: {}", e))?;

        // 6. Merger RSI + Stochastic + EMA + Point Pivot dans un seul DataFrame
        let df_with_indicators = self.merge_indicators(df_new_dates, df_rsi, df_stoch, df_ema, df_pivot)?;

        // 7. UPSERT batch
        let inserted = self.upsert_indicators(&df_with_indicators, db).await?;
        println!("✅ FLUX A: Saved {} records", inserted);

        Ok(inserted)
    }

    /// UPSERT batch dans indicators_test (pour FLUX A)
    async fn upsert_indicators(&self, df: &DataFrame, db: &DatabaseConnection) -> Result<usize, String> {
        println!("💾 Preparing batch UPSERT for {} rows...", df.height());

        // ============================================================================
        // VERSION VM GRATUITE : UPSERT PAR SYMBOLE AVEC TRANSACTIONS (100% SeaORM)
        // ============================================================================
        self.upsert_by_symbol_seaorm(df, db).await

        // ============================================================================
        // VERSION VM PAYANTE : BATCH UPSERT AVEC SQLX (décommenter quand VM performante)
        // Utilise sqlx pour faire des batch INSERT massifs en une seule query
        // ============================================================================
        // self.upsert_batch_sqlx(df, db).await
    }

    /// Récupère historicdata après une date (pour FLUX A)
    async fn fetch_historicdata_after(&self, cutoff: &str, symbols: &[String], db: &DatabaseConnection) -> Result<DataFrame, String> {
        let historical_data = HistoricData::find()
            .filter(historic_data::Column::Date.gt(cutoff))
            .filter(historic_data::Column::Symbol.is_in(symbols.iter().map(|s| s.as_str())))
            .order_by_asc(historic_data::Column::Symbol)
            .order_by_asc(historic_data::Column::Date)
            .all(db)
            .await
            .map_err(|e| format!("Failed to fetch historical data: {}", e))?;

        self.convert_to_dataframe(historical_data)
    }

    /// FLUX B : Traite les nouveaux symboles (full)
    async fn process_new_symbols(&self, new_symbols: &[String], db: &DatabaseConnection) -> Result<usize, String> {
        println!("🔄 FLUX B: Processing {} new symbols (full calculation)", new_symbols.len());

        // 1. Fetch TOUTES les données pour ces symboles
        let df_all = self.fetch_all_for_symbols(new_symbols, db).await?;
        println!("📊 df_all: {} rows", df_all.height());

        if df_all.height() == 0 {
            println!("⚠️  No historical data for new symbols");
            return Ok(0);
        }

        // 2. Calculer RSI + Stochastic + EMA + Point Pivot (df_full = df_new car tout est nouveau)
        let rsi_calculator = RSICalculator::new(25);
        let stoch_calculator = StochasticCalculator::new(14, 7, 7);
        let ema_calculator = EMACalculator::new(vec![20, 50, 200]);
        let pivot_calculator = PointPivotCalculator::new();

        let df_rsi = rsi_calculator.calculate(df_all.clone(), &df_all)
            .map_err(|e| format!("RSI calculation error: {}", e))?;

        let df_stoch = stoch_calculator.calculate(df_all.clone(), &df_all)
            .map_err(|e| format!("Stochastic calculation error: {}", e))?;

        let df_ema = ema_calculator.calculate(df_all.clone(), &df_all)
            .map_err(|e| format!("EMA calculation error: {}", e))?;

        let df_pivot = pivot_calculator.calculate(df_all.clone(), &df_all)
            .map_err(|e| format!("Point Pivot calculation error: {}", e))?;

        // 3. Merger RSI + Stochastic + EMA + Point Pivot dans un seul DataFrame
        let df_with_indicators = self.merge_indicators(df_all, df_rsi, df_stoch, df_ema, df_pivot)?;

        // 4. INSERT batch (pas d'UPSERT car nouveaux symboles)
        let inserted = self.insert_indicators(&df_with_indicators, db).await?;
        println!("✅ FLUX B: Saved {} records", inserted);

        Ok(inserted)
    }

    /// INSERT batch dans indicators_test (pour FLUX B)
    async fn insert_indicators(&self, df: &DataFrame, db: &DatabaseConnection) -> Result<usize, String> {
        println!("💾 Preparing batch INSERT for {} rows...", df.height());

        // ============================================================================
        // VERSION VM GRATUITE : INSERT PAR SYMBOLE AVEC TRANSACTIONS (100% SeaORM)
        // ============================================================================
        self.insert_by_symbol_seaorm(df, db).await

        // ============================================================================
        // VERSION VM PAYANTE : BATCH INSERT AVEC SQLX (décommenter quand VM performante)
        // Utilise sqlx pour faire des batch INSERT massifs en une seule query
        // ============================================================================
        // self.insert_batch_sqlx(df, db).await
    }

    /// Récupère TOUTES les données pour des symboles spécifiques (pour FLUX B)
    async fn fetch_all_for_symbols(&self, symbols: &[String], db: &DatabaseConnection) -> Result<DataFrame, String> {
        let historical_data = HistoricData::find()
            .filter(historic_data::Column::Symbol.is_in(symbols.iter().map(|s| s.as_str())))
            .order_by_asc(historic_data::Column::Symbol)
            .order_by_asc(historic_data::Column::Date)
            .all(db)
            .await
            .map_err(|e| format!("Failed to fetch historical data: {}", e))?;

        self.convert_to_dataframe(historical_data)
    }

    /// Convertit Vec<HistoricDataModel> en DataFrame polars
    fn convert_to_dataframe(&self, historical_data: Vec<historic_data::Model>) -> Result<DataFrame, String> {
        let mut dates = Vec::new();
        let mut symbols = Vec::new();
        let mut opens = Vec::new();
        let mut highs = Vec::new();
        let mut lows = Vec::new();
        let mut closes = Vec::new();

        for data in historical_data {
            if let (Some(open_str), Some(high_str), Some(low_str), Some(close_str)) =
                (&data.open, &data.high, &data.low, &data.close)
            {
                if let (Ok(open), Ok(high), Ok(low), Ok(close)) = (
                    open_str.parse::<f64>(),
                    high_str.parse::<f64>(),
                    low_str.parse::<f64>(),
                    close_str.parse::<f64>(),
                ) {
                    dates.push(data.date.clone());
                    symbols.push(data.symbol.clone());
                    opens.push(open);
                    highs.push(high);
                    lows.push(low);
                    closes.push(close);
                }
            }
        }

        DataFrame::new(vec![
            Column::Series(Series::new("date".into(), dates)),
            Column::Series(Series::new("symbol".into(), symbols)),
            Column::Series(Series::new("open".into(), opens)),
            Column::Series(Series::new("high".into(), highs)),
            Column::Series(Series::new("low".into(), lows)),
            Column::Series(Series::new("close".into(), closes)),
        ]).map_err(|e| format!("Failed to create DataFrame: {}", e))
    }

    /// Merge RSI + Stochastic + EMA + Point Pivot dans un seul DataFrame
    fn merge_indicators(
        &self,
        df_base: DataFrame,
        df_rsi: DataFrame,
        df_stoch: DataFrame,
        df_ema: DataFrame,
        df_pivot: DataFrame,
    ) -> Result<DataFrame, String> {
        println!("🔗 Merging indicators...");

        let date_col = df_base.column("date").map_err(|e| format!("Failed to get date: {}", e))?;
        let symbol_col = df_base.column("symbol").map_err(|e| format!("Failed to get symbol: {}", e))?;

        let rsi_col = df_rsi.column("rsi25").map_err(|e| format!("Failed to get rsi25: {}", e))?;
        let stoch_col = df_stoch.column("stochastic14_7_7").map_err(|e| format!("Failed to get stochastic14_7_7: {}", e))?;
        let ema20_col = df_ema.column("ema20").map_err(|e| format!("Failed to get ema20: {}", e))?;
        let ema50_col = df_ema.column("ema50").map_err(|e| format!("Failed to get ema50: {}", e))?;
        let ema200_col = df_ema.column("ema200").map_err(|e| format!("Failed to get ema200: {}", e))?;
        let pivot_col = df_pivot.column("point_pivot").map_err(|e| format!("Failed to get point_pivot: {}", e))?;

        let mut dates = Vec::new();
        let mut symbols = Vec::new();
        let mut rsis = Vec::new();
        let mut stochs = Vec::new();
        let mut ema20s = Vec::new();
        let mut ema50s = Vec::new();
        let mut ema200s = Vec::new();
        let mut pivots = Vec::new();

        for i in 0..df_base.height() {
            let date = match date_col.get(i).map_err(|e| format!("Get date error: {}", e))? {
                AnyValue::String(s) => s.to_string(),
                val => val.to_string().replace('"', ""),
            };

            let symbol = match symbol_col.get(i).map_err(|e| format!("Get symbol error: {}", e))? {
                AnyValue::String(s) => s.to_string(),
                val => val.to_string().replace('"', ""),
            };

            let rsi = rsi_col.get(i).ok();
            let stoch = stoch_col.get(i).ok();
            let ema20 = ema20_col.get(i).ok();
            let ema50 = ema50_col.get(i).ok();
            let ema200 = ema200_col.get(i).ok();
            let pivot = pivot_col.get(i).ok();

            dates.push(date);
            symbols.push(symbol);
            rsis.push(if let Some(AnyValue::Float64(v)) = rsi { Some(v) } else { None });
            stochs.push(if let Some(AnyValue::Float64(v)) = stoch { Some(v) } else { None });
            ema20s.push(if let Some(AnyValue::Float64(v)) = ema20 { Some(v) } else { None });
            ema50s.push(if let Some(AnyValue::Float64(v)) = ema50 { Some(v) } else { None });
            ema200s.push(if let Some(AnyValue::Float64(v)) = ema200 { Some(v) } else { None });
            pivots.push(if let Some(AnyValue::String(s)) = pivot { Some(s.to_string()) } else { None });
        }

        let result = DataFrame::new(vec![
            Column::Series(Series::new("date".into(), dates)),
            Column::Series(Series::new("symbol".into(), symbols)),
            Column::Series(Series::new("rsi25".into(), rsis)),
            Column::Series(Series::new("stochastic14_7_7".into(), stochs)),
            Column::Series(Series::new("ema20".into(), ema20s)),
            Column::Series(Series::new("ema50".into(), ema50s)),
            Column::Series(Series::new("ema200".into(), ema200s)),
            Column::Series(Series::new("point_pivot".into(), pivots)),
        ]).map_err(|e| format!("Failed to create merged DataFrame: {}", e))?;

        println!("✅ Merged DataFrame: {} rows", result.height());
        Ok(result)
    }

    // ============================================================================
    // MÉTHODES VM GRATUITE (100% SeaORM avec transactions par symbole)
    // ============================================================================

    /// UPSERT par symbole avec transactions SeaORM (VM gratuite)
    async fn upsert_by_symbol_seaorm(&self, df: &DataFrame, db: &DatabaseConnection) -> Result<usize, String> {
        let date_col = df.column("date").map_err(|e| format!("Failed to get date: {}", e))?;
        let symbol_col = df.column("symbol").map_err(|e| format!("Failed to get symbol: {}", e))?;
        let rsi_col = df.column("rsi25").map_err(|e| format!("Failed to get rsi25: {}", e))?;
        let stoch_col = df.column("stochastic14_7_7").map_err(|e| format!("Failed to get stochastic14_7_7: {}", e))?;
        let ema20_col = df.column("ema20").map_err(|e| format!("Failed to get ema20: {}", e))?;
        let ema50_col = df.column("ema50").map_err(|e| format!("Failed to get ema50: {}", e))?;
        let ema200_col = df.column("ema200").map_err(|e| format!("Failed to get ema200: {}", e))?;
        let pivot_col = df.column("point_pivot").map_err(|e| format!("Failed to get point_pivot: {}", e))?;

        // Grouper par symbole
        let mut symbol_data: std::collections::HashMap<String, Vec<(String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)>> = std::collections::HashMap::new();

        for i in 0..df.height() {
            let date = match date_col.get(i).map_err(|e| format!("Get date error: {}", e))? {
                AnyValue::String(s) => s.to_string(),
                val => val.to_string().replace('"', ""),
            };

            let symbol = match symbol_col.get(i).map_err(|e| format!("Get symbol error: {}", e))? {
                AnyValue::String(s) => s.to_string(),
                val => val.to_string().replace('"', ""),
            };

            let rsi_value = rsi_col.get(i).map_err(|e| format!("Get RSI error: {}", e))?;
            let stoch_value = stoch_col.get(i).map_err(|e| format!("Get Stochastic error: {}", e))?;
            let ema20_value = ema20_col.get(i).map_err(|e| format!("Get EMA20 error: {}", e))?;
            let ema50_value = ema50_col.get(i).map_err(|e| format!("Get EMA50 error: {}", e))?;
            let ema200_value = ema200_col.get(i).map_err(|e| format!("Get EMA200 error: {}", e))?;
            let pivot_value = pivot_col.get(i).map_err(|e| format!("Get Point Pivot error: {}", e))?;

            let rsi_str = if !rsi_value.is_null() {
                Some(match rsi_value {
                    AnyValue::Float64(f) => format!("{:.2}", f),
                    val => val.to_string().replace('"', ""),
                })
            } else {
                None
            };

            let stoch_str = if !stoch_value.is_null() {
                Some(match stoch_value {
                    AnyValue::Float64(f) => format!("{:.2}", f),
                    val => val.to_string().replace('"', ""),
                })
            } else {
                None
            };

            let ema20_str = if !ema20_value.is_null() {
                Some(match ema20_value {
                    AnyValue::Float64(f) => format!("{:.2}", f),
                    val => val.to_string().replace('"', ""),
                })
            } else {
                None
            };

            let ema50_str = if !ema50_value.is_null() {
                Some(match ema50_value {
                    AnyValue::Float64(f) => format!("{:.2}", f),
                    val => val.to_string().replace('"', ""),
                })
            } else {
                None
            };

            let ema200_str = if !ema200_value.is_null() {
                Some(match ema200_value {
                    AnyValue::Float64(f) => format!("{:.2}", f),
                    val => val.to_string().replace('"', ""),
                })
            } else {
                None
            };

            let pivot_str = if !pivot_value.is_null() {
                Some(match pivot_value {
                    AnyValue::String(s) => s.to_string(),
                    val => val.to_string().replace('"', ""),
                })
            } else {
                None
            };

            // Insérer seulement si au moins un indicateur n'est pas null
            if rsi_str.is_some() || stoch_str.is_some() || ema20_str.is_some() || ema50_str.is_some() || ema200_str.is_some() || pivot_str.is_some() {
                symbol_data.entry(symbol).or_insert_with(Vec::new).push((date, rsi_str, stoch_str, ema20_str, ema50_str, ema200_str, pivot_str));
            }
        }

        let total_symbols = symbol_data.len();
        let mut total_inserted = 0;

        // Traiter chaque symbole dans sa propre transaction
        for (symbol_idx, (symbol, rows)) in symbol_data.iter().enumerate() {
            let txn = db.begin().await.map_err(|e| format!("Transaction begin error: {}", e))?;

            for (date, rsi, stoch, ema20, ema50, ema200, pivot) in rows {
                // Chercher si existe
                let existing = Indicator::find()
                    .filter(IndicatorColumn::Date.eq(date))
                    .filter(IndicatorColumn::Symbol.eq(symbol))
                    .one(&txn)
                    .await
                    .map_err(|e| format!("Query error: {}", e))?;

                match existing {
                    Some(model) => {
                        // UPDATE
                        let mut active: IndicatorActiveModel = model.into();
                        active.rsi25 = Set(rsi.clone());
                        active.stochastic14_7_7 = Set(stoch.clone());
                        active.ema20 = Set(ema20.clone());
                        active.ema50 = Set(ema50.clone());
                        active.ema200 = Set(ema200.clone());

                        // Convertir pivot_str en serde_json::Value
                        active.point_pivot = Set(pivot.as_ref().and_then(|s| serde_json::from_str(s).ok()));

                        active.update(&txn).await.map_err(|e| format!("Update error: {}", e))?;
                    }
                    None => {
                        // INSERT
                        let new = IndicatorActiveModel {
                            date: Set(date.clone()),
                            symbol: Set(symbol.clone()),
                            rsi25: Set(rsi.clone()),
                            stochastic14_7_7: Set(stoch.clone()),
                            ema20: Set(ema20.clone()),
                            ema50: Set(ema50.clone()),
                            ema200: Set(ema200.clone()),
                            point_pivot: Set(pivot.as_ref().and_then(|s| serde_json::from_str(s).ok())),
                            ..Default::default()
                        };
                        new.insert(&txn).await.map_err(|e| format!("Insert error: {}", e))?;
                    }
                }
            }

            txn.commit().await.map_err(|e| format!("Transaction commit error: {}", e))?;

            total_inserted += rows.len();
            println!("💾 UPSERT: Symbol {}/{} completed - {} ({} rows)", symbol_idx + 1, total_symbols, symbol, rows.len());
        }

        println!("✅ Batch UPSERT completed: {} rows total", total_inserted);
        Ok(total_inserted)
    }

    /// INSERT par symbole avec transactions SeaORM (VM gratuite)
    async fn insert_by_symbol_seaorm(&self, df: &DataFrame, db: &DatabaseConnection) -> Result<usize, String> {
        let date_col = df.column("date").map_err(|e| format!("Failed to get date: {}", e))?;
        let symbol_col = df.column("symbol").map_err(|e| format!("Failed to get symbol: {}", e))?;
        let rsi_col = df.column("rsi25").map_err(|e| format!("Failed to get rsi25: {}", e))?;
        let stoch_col = df.column("stochastic14_7_7").map_err(|e| format!("Failed to get stochastic14_7_7: {}", e))?;
        let ema20_col = df.column("ema20").map_err(|e| format!("Failed to get ema20: {}", e))?;
        let ema50_col = df.column("ema50").map_err(|e| format!("Failed to get ema50: {}", e))?;
        let ema200_col = df.column("ema200").map_err(|e| format!("Failed to get ema200: {}", e))?;
        let pivot_col = df.column("point_pivot").map_err(|e| format!("Failed to get point_pivot: {}", e))?;

        // Grouper par symbole
        let mut symbol_data: std::collections::HashMap<String, Vec<(String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)>> = std::collections::HashMap::new();

        for i in 0..df.height() {
            let date = match date_col.get(i).map_err(|e| format!("Get date error: {}", e))? {
                AnyValue::String(s) => s.to_string(),
                val => val.to_string().replace('"', ""),
            };

            let symbol = match symbol_col.get(i).map_err(|e| format!("Get symbol error: {}", e))? {
                AnyValue::String(s) => s.to_string(),
                val => val.to_string().replace('"', ""),
            };

            let rsi_value = rsi_col.get(i).map_err(|e| format!("Get RSI error: {}", e))?;
            let stoch_value = stoch_col.get(i).map_err(|e| format!("Get Stochastic error: {}", e))?;
            let ema20_value = ema20_col.get(i).map_err(|e| format!("Get EMA20 error: {}", e))?;
            let ema50_value = ema50_col.get(i).map_err(|e| format!("Get EMA50 error: {}", e))?;
            let ema200_value = ema200_col.get(i).map_err(|e| format!("Get EMA200 error: {}", e))?;
            let pivot_value = pivot_col.get(i).map_err(|e| format!("Get Point Pivot error: {}", e))?;

            let rsi_str = if !rsi_value.is_null() {
                Some(match rsi_value {
                    AnyValue::Float64(f) => format!("{:.2}", f),
                    val => val.to_string().replace('"', ""),
                })
            } else {
                None
            };

            let stoch_str = if !stoch_value.is_null() {
                Some(match stoch_value {
                    AnyValue::Float64(f) => format!("{:.2}", f),
                    val => val.to_string().replace('"', ""),
                })
            } else {
                None
            };

            let ema20_str = if !ema20_value.is_null() {
                Some(match ema20_value {
                    AnyValue::Float64(f) => format!("{:.2}", f),
                    val => val.to_string().replace('"', ""),
                })
            } else {
                None
            };

            let ema50_str = if !ema50_value.is_null() {
                Some(match ema50_value {
                    AnyValue::Float64(f) => format!("{:.2}", f),
                    val => val.to_string().replace('"', ""),
                })
            } else {
                None
            };

            let ema200_str = if !ema200_value.is_null() {
                Some(match ema200_value {
                    AnyValue::Float64(f) => format!("{:.2}", f),
                    val => val.to_string().replace('"', ""),
                })
            } else {
                None
            };

            let pivot_str = if !pivot_value.is_null() {
                Some(match pivot_value {
                    AnyValue::String(s) => s.to_string(),
                    val => val.to_string().replace('"', ""),
                })
            } else {
                None
            };

            // Insérer seulement si au moins un indicateur n'est pas null
            if rsi_str.is_some() || stoch_str.is_some() || ema20_str.is_some() || ema50_str.is_some() || ema200_str.is_some() || pivot_str.is_some() {
                symbol_data.entry(symbol).or_insert_with(Vec::new).push((date, rsi_str, stoch_str, ema20_str, ema50_str, ema200_str, pivot_str));
            }
        }

        let total_symbols = symbol_data.len();
        let mut total_inserted = 0;

        // Traiter chaque symbole dans sa propre transaction
        for (symbol_idx, (symbol, rows)) in symbol_data.iter().enumerate() {
            let txn = db.begin().await.map_err(|e| format!("Transaction begin error: {}", e))?;

            for (date, rsi, stoch, ema20, ema50, ema200, pivot) in rows {
                let new = IndicatorActiveModel {
                    date: Set(date.clone()),
                    symbol: Set(symbol.clone()),
                    rsi25: Set(rsi.clone()),
                    stochastic14_7_7: Set(stoch.clone()),
                    ema20: Set(ema20.clone()),
                    ema50: Set(ema50.clone()),
                    ema200: Set(ema200.clone()),
                    point_pivot: Set(pivot.as_ref().and_then(|s| serde_json::from_str(s).ok())),
                    ..Default::default()
                };
                new.insert(&txn).await.map_err(|e| format!("Insert error: {}", e))?;
            }

            txn.commit().await.map_err(|e| format!("Transaction commit error: {}", e))?;

            total_inserted += rows.len();
            println!("💾 INSERT: Symbol {}/{} completed - {} ({} rows)", symbol_idx + 1, total_symbols, symbol, rows.len());
        }

        println!("✅ Batch INSERT completed: {} rows total", total_inserted);
        Ok(total_inserted)
    }

    // ============================================================================
    // MÉTHODES VM PAYANTE (BATCH SQLX) - COMMENTÉES
    // Décommenter ces méthodes et commenter les appels ci-dessus quand VM performante
    // ============================================================================

    /*
    /// UPSERT batch avec sqlx (VM payante) - Ultra rapide avec chunks
    async fn upsert_batch_sqlx(&self, df: &DataFrame, db: &DatabaseConnection) -> Result<usize, String> {
        // TODO: Adapter pour inclure tous les indicateurs
        unimplemented!("SQLX batch upsert not yet implemented for all indicators")
    }

    /// INSERT batch avec sqlx (VM payante) - Ultra rapide avec chunks
    async fn insert_batch_sqlx(&self, df: &DataFrame, db: &DatabaseConnection) -> Result<usize, String> {
        // TODO: Adapter pour inclure tous les indicateurs
        unimplemented!("SQLX batch insert not yet implemented for all indicators")
    }
    */
}