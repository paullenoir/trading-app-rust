use polars::prelude::*;
use std::collections::HashMap;

pub struct EMACalculator {
    periods: Vec<usize>, // [20, 50, 200]
}

impl EMACalculator {
    pub fn new(periods: Vec<usize>) -> Self {
        Self { periods }
    }

    pub fn calculate(
        &self,
        df_new: DataFrame,
        df_full: &DataFrame,
    ) -> Result<DataFrame, PolarsError> {
        println!("🔄 Calculating EMA for {} rows", df_new.height());

        // 1. Grouper df_full par symbole
        let grouped_full = self.group_by_symbol(df_full)?;

        println!("📊 EMA: Grouped {} unique symbols", grouped_full.len());

        // 2. Calculer EMA pour chaque période et chaque symbole
        let mut ema_results: HashMap<(String, String, usize), f64> = HashMap::new();

        let mut symbol_idx = 0;
        let total_symbols = grouped_full.len();

        for (symbol, closes_with_dates) in grouped_full.iter() {
            symbol_idx += 1;
            println!("📊 EMA: Processing symbol {}/{}: {}", symbol_idx, total_symbols, symbol);

            // Calculer EMA pour chaque période
            for &period in &self.periods {
                let ema_values = self.compute_ema(&closes_with_dates, period);

                for (i, ema) in ema_values.iter().enumerate() {
                    if let Some(ema_val) = ema {
                        let date = &closes_with_dates[i].0;
                        ema_results.insert((symbol.clone(), date.clone(), period), *ema_val);
                    }
                }
            }
        }

        println!("✅ EMA: Calculated {} values", ema_results.len());

        // 3. Construire le DataFrame résultat avec seulement df_new
        let date_col = df_new.column("date")?;
        let symbol_col = df_new.column("symbol")?;

        let mut dates = Vec::new();
        let mut symbols = Vec::new();
        let mut ema20s = Vec::new();
        let mut ema50s = Vec::new();
        let mut ema200s = Vec::new();

        for i in 0..df_new.height() {
            let date = date_col.get(i)?.to_string();
            let symbol = symbol_col.get(i)?.to_string();

            let ema20 = ema_results.get(&(symbol.clone(), date.clone(), 20)).copied();
            let ema50 = ema_results.get(&(symbol.clone(), date.clone(), 50)).copied();
            let ema200 = ema_results.get(&(symbol.clone(), date.clone(), 200)).copied();

            dates.push(date);
            symbols.push(symbol);
            ema20s.push(ema20);
            ema50s.push(ema50);
            ema200s.push(ema200);
        }

        let result = DataFrame::new(vec![
            Column::Series(Series::new("date".into(), dates)),
            Column::Series(Series::new("symbol".into(), symbols)),
            Column::Series(Series::new("ema20".into(), ema20s)),
            Column::Series(Series::new("ema50".into(), ema50s)),
            Column::Series(Series::new("ema200".into(), ema200s)),
        ])?;

        println!("✅ EMA: Result DataFrame has {} rows", result.height());
        Ok(result)
    }

    /// Groupe df par symbole et retourne HashMap<symbol, Vec<(date, close)>>
    fn group_by_symbol(&self, df: &DataFrame) -> Result<HashMap<String, Vec<(String, f64)>>, PolarsError> {
        let date_col = df.column("date")?;
        let symbol_col = df.column("symbol")?;
        let close_col = df.column("close")?;

        let mut grouped: HashMap<String, Vec<(String, f64)>> = HashMap::new();

        for i in 0..df.height() {
            let date = date_col.get(i)?.to_string();
            let symbol = symbol_col.get(i)?.to_string();
            let close = if let AnyValue::Float64(v) = close_col.get(i)? { v } else { continue };

            grouped.entry(symbol).or_insert_with(Vec::new).push((date, close));
        }

        Ok(grouped)
    }

    /// Calcule l'EMA pour une période donnée
    /// Retourne Vec<Option<f64>> de même longueur que closes_with_dates
    fn compute_ema(&self, closes_with_dates: &[(String, f64)], period: usize) -> Vec<Option<f64>> {
        let mut ema_values = Vec::new();

        if closes_with_dates.len() < period {
            // Pas assez de données
            return vec![None; closes_with_dates.len()];
        }

        let multiplier = 2.0 / (period as f64 + 1.0);

        // Calculer la SMA initiale (Simple Moving Average) pour les 'period' premières valeurs
        let initial_sma: f64 = closes_with_dates[0..period]
            .iter()
            .map(|(_, close)| close)
            .sum::<f64>() / period as f64;

        // Remplir les None pour les valeurs avant la période
        for _ in 0..(period - 1) {
            ema_values.push(None);
        }

        // La première EMA est la SMA
        ema_values.push(Some(initial_sma));
        let mut previous_ema = initial_sma;

        // Calculer les EMA suivantes
        for i in period..closes_with_dates.len() {
            let close = closes_with_dates[i].1;
            let ema = (close * multiplier) + (previous_ema * (1.0 - multiplier));
            ema_values.push(Some(ema));
            previous_ema = ema;
        }

        ema_values
    }
}