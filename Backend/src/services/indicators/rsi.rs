use polars::prelude::*;
use std::collections::HashMap;

pub struct RSICalculator {
    period: usize,
}

impl RSICalculator {
    pub fn new(period: usize) -> Self {
        Self { period }
    }

    pub fn calculate(
        &self,
        df_new: DataFrame,
        df_full: &DataFrame,
    ) -> Result<DataFrame, PolarsError> {
        println!("🔄 Calculating RSI for {} rows", df_new.height());

        // 1. Grouper df_full par symbole (une seule fois)
        let grouped_full = self.group_by_symbol(df_full)?;

        println!("📊 RSI: Grouped {} unique symbols", grouped_full.len());

        // 2. Calculer RSI pour chaque symbole
        let mut rsi_results: HashMap<(String, String), f64> = HashMap::new();

        let mut symbol_idx = 0;
        let total_symbols = grouped_full.len();

        for (symbol, closes_with_dates) in grouped_full.iter() {
            symbol_idx += 1;
            println!("📊 RSI: Processing symbol {}/{}: {}", symbol_idx, total_symbols, symbol);

            // Calculer RSI pour ce symbole
            for i in 0..closes_with_dates.len() {
                if i > self.period {
                    let window = &closes_with_dates[i - self.period..=i];
                    let closes: Vec<f64> = window.iter().map(|(_, c)| *c).collect();

                    if let Some(rsi) = self.compute_rsi(&closes) {
                        let date = &closes_with_dates[i].0;
                        rsi_results.insert((symbol.clone(), date.clone()), rsi);
                    }
                }
            }
        }

        println!("✅ RSI: Calculated {} values", rsi_results.len());

        // 3. Construire le DataFrame résultat avec seulement df_new
        let date_col = df_new.column("date")?;
        let symbol_col = df_new.column("symbol")?;
        let close_col = df_new.column("close")?;

        let mut dates = Vec::new();
        let mut symbols = Vec::new();
        let mut closes = Vec::new();
        let mut rsis = Vec::new();

        for i in 0..df_new.height() {
            let date = date_col.get(i)?.to_string();
            let symbol = symbol_col.get(i)?.to_string();
            let close = if let AnyValue::Float64(v) = close_col.get(i)? { v } else { 0.0 };

            let rsi = rsi_results.get(&(symbol.clone(), date.clone())).copied();

            dates.push(date);
            symbols.push(symbol);
            closes.push(close);
            rsis.push(rsi);
        }

        let result = DataFrame::new(vec![
            Column::Series(Series::new("date".into(), dates)),
            Column::Series(Series::new("symbol".into(), symbols)),
            Column::Series(Series::new("close".into(), closes)),
            Column::Series(Series::new("rsi25".into(), rsis)),
        ])?;

        println!("✅ RSI: Result DataFrame has {} rows", result.height());
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

    fn compute_rsi(&self, closes: &[f64]) -> Option<f64> {
        if closes.len() <= self.period {
            return None;
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..closes.len() {
            let change = closes[i] - closes[i - 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        // Prendre les derniers 'period' gains/losses
        let recent_gains = &gains[gains.len().saturating_sub(self.period)..];
        let recent_losses = &losses[losses.len().saturating_sub(self.period)..];

        let avg_gain: f64 = recent_gains.iter().sum::<f64>() / self.period as f64;
        let avg_loss: f64 = recent_losses.iter().sum::<f64>() / self.period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        Some(rsi)
    }
}