use polars::prelude::*;
use std::collections::HashMap;

pub struct StochasticCalculator {
    k_period: usize,      // 14 pour le min/max
    k_slowing: usize,     // 7 pour la moyenne du %K
    d_period: usize,      // 7 pour la moyenne du %D (non utilisé ici)
}

impl StochasticCalculator {
    pub fn new(k_period: usize, k_slowing: usize, d_period: usize) -> Self {
        Self {
            k_period,
            k_slowing,
            d_period,
        }
    }

    pub fn calculate(
        &self,
        df_new: DataFrame,
        df_full: &DataFrame,
    ) -> Result<DataFrame, PolarsError> {
        println!("🔄 Calculating Stochastic for {} rows", df_new.height());

        // 1. Grouper df_full par symbole
        let grouped_full = self.group_by_symbol(df_full)?;

        println!("📊 STOCHASTIC: Grouped {} unique symbols", grouped_full.len());

        // 2. Calculer Stochastic pour chaque symbole
        let mut stoch_results: HashMap<(String, String), f64> = HashMap::new();

        let mut symbol_idx = 0;
        let total_symbols = grouped_full.len();

        for (symbol, data) in grouped_full.iter() {
            symbol_idx += 1;
            println!("📊 STOCHASTIC: Processing symbol {}/{}: {}", symbol_idx, total_symbols, symbol);

            // Calculer Stochastic pour ce symbole
            for i in 0..data.len() {
                // Besoin de k_period + k_slowing périodes minimum
                let min_periods = self.k_period + self.k_slowing - 1;

                if i >= min_periods {
                    // Window pour min/max (14 périodes)
                    let window_minmax = &data[i - self.k_period + 1..=i];

                    if self.compute_fast_k(window_minmax).is_some() {
                        // Window pour moyenne mobile du %K (7 périodes)
                        if i >= self.k_period + self.k_slowing - 2 {
                            let mut fast_k_values = Vec::new();

                            for j in (i - self.k_slowing + 1)..=i {
                                let win = &data[j - self.k_period + 1..=j];
                                if let Some(fk) = self.compute_fast_k(win) {
                                    fast_k_values.push(fk);
                                }
                            }

                            if fast_k_values.len() == self.k_slowing {
                                let stoch = fast_k_values.iter().sum::<f64>() / self.k_slowing as f64;
                                let date = &data[i].0;
                                stoch_results.insert((symbol.clone(), date.clone()), stoch);
                            }
                        }
                    }
                }
            }
        }

        println!("✅ STOCHASTIC: Calculated {} values", stoch_results.len());

        // 3. Construire le DataFrame résultat avec seulement df_new
        let date_col = df_new.column("date")?;
        let symbol_col = df_new.column("symbol")?;

        let mut dates = Vec::new();
        let mut symbols = Vec::new();
        let mut stochs = Vec::new();

        for i in 0..df_new.height() {
            let date = date_col.get(i)?.to_string();
            let symbol = symbol_col.get(i)?.to_string();

            let stoch = stoch_results.get(&(symbol.clone(), date.clone())).copied();

            dates.push(date);
            symbols.push(symbol);
            stochs.push(stoch);
        }

        let result = DataFrame::new(vec![
            Column::Series(Series::new("date".into(), dates)),
            Column::Series(Series::new("symbol".into(), symbols)),
            Column::Series(Series::new("stochastic14_7_7".into(), stochs)),
        ])?;

        println!("✅ STOCHASTIC: Result DataFrame has {} rows", result.height());
        Ok(result)
    }

    /// Groupe df par symbole et retourne HashMap<symbol, Vec<(date, high, low, close)>>
    fn group_by_symbol(&self, df: &DataFrame) -> Result<HashMap<String, Vec<(String, f64, f64, f64)>>, PolarsError> {
        let date_col = df.column("date")?;
        let symbol_col = df.column("symbol")?;
        let high_col = df.column("high")?;
        let low_col = df.column("low")?;
        let close_col = df.column("close")?;

        let mut grouped: HashMap<String, Vec<(String, f64, f64, f64)>> = HashMap::new();

        for i in 0..df.height() {
            let date = date_col.get(i)?.to_string();
            let symbol = symbol_col.get(i)?.to_string();
            let high = if let AnyValue::Float64(v) = high_col.get(i)? { v } else { continue };
            let low = if let AnyValue::Float64(v) = low_col.get(i)? { v } else { continue };
            let close = if let AnyValue::Float64(v) = close_col.get(i)? { v } else { continue };

            grouped.entry(symbol).or_insert_with(Vec::new).push((date, high, low, close));
        }

        Ok(grouped)
    }

    /// Calcule le Fast %K pour une window donnée
    /// Fast %K = 100 * (close - lowest_low) / (highest_high - lowest_low)
    fn compute_fast_k(&self, window: &[(String, f64, f64, f64)]) -> Option<f64> {
        if window.is_empty() {
            return None;
        }

        let lowest_low = window.iter().map(|(_, _, low, _)| *low).fold(f64::INFINITY, f64::min);
        let highest_high = window.iter().map(|(_, high, _, _)| *high).fold(f64::NEG_INFINITY, f64::max);
        let current_close = window.last()?.3;

        let denominator = highest_high - lowest_low;
        if denominator == 0.0 {
            return Some(0.0);
        }

        let fast_k = 100.0 * (current_close - lowest_low) / denominator;
        Some(fast_k)
    }
}