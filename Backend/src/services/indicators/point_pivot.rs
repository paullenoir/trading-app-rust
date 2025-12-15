use polars::prelude::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
struct CamarillaPivot {
    pivot: f64,
    r1: f64,
    r2: f64,
    r3: f64,
    s1: f64,
    s2: f64,
    s3: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct AllPivots {
    #[serde(skip_serializing_if = "Option::is_none")]
    week: Option<CamarillaPivot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    month: Option<CamarillaPivot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<CamarillaPivot>,
}

pub struct PointPivotCalculator;

impl PointPivotCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate(
        &self,
        df_new: DataFrame,
        df_full: &DataFrame,
    ) -> Result<DataFrame, PolarsError> {
        println!("🔄 Calculating Point Pivot for {} rows", df_new.height());

        // 1. Grouper df_full par symbole
        let grouped_full = self.group_by_symbol(df_full)?;

        println!("📊 POINT PIVOT: Grouped {} unique symbols", grouped_full.len());

        // 2. Calculer les points pivots pour chaque symbole
        let mut pivot_results: HashMap<(String, String), String> = HashMap::new();

        let mut symbol_idx = 0;
        let total_symbols = grouped_full.len();

        for (symbol, data) in grouped_full.iter() {
            symbol_idx += 1;
            println!("📊 POINT PIVOT: Processing symbol {}/{}: {}", symbol_idx, total_symbols, symbol);

            // Pour chaque date dans les données du symbole
            for i in 0..data.len() {
                let current_date = &data[i].0;

                // Calculer week pivots (7 derniers jours)
                let week_pivots = self.calculate_period_pivots(data, i, 7, 2);

                // Calculer month pivots (30 derniers jours)
                let month_pivots = self.calculate_period_pivots(data, i, 30, 5);

                // Calculer year pivots (365 derniers jours)
                let year_pivots = self.calculate_period_pivots(data, i, 365, 30);

                // Si au moins un pivot existe, créer le JSON
                if week_pivots.is_some() || month_pivots.is_some() || year_pivots.is_some() {
                    let all_pivots = AllPivots {
                        week: week_pivots,
                        month: month_pivots,
                        year: year_pivots,
                    };

                    if let Ok(json_str) = serde_json::to_string(&all_pivots) {
                        pivot_results.insert((symbol.clone(), current_date.clone()), json_str);
                    }
                }
            }
        }

        println!("✅ POINT PIVOT: Calculated {} values", pivot_results.len());

        // 3. Construire le DataFrame résultat avec seulement df_new
        let date_col = df_new.column("date")?;
        let symbol_col = df_new.column("symbol")?;

        let mut dates = Vec::new();
        let mut symbols = Vec::new();
        let mut pivots = Vec::new();

        for i in 0..df_new.height() {
            let date = date_col.get(i)?.to_string();
            let symbol = symbol_col.get(i)?.to_string();

            let pivot = pivot_results.get(&(symbol.clone(), date.clone())).cloned();

            dates.push(date);
            symbols.push(symbol);
            pivots.push(pivot);
        }

        let result = DataFrame::new(vec![
            Column::Series(Series::new("date".into(), dates)),
            Column::Series(Series::new("symbol".into(), symbols)),
            Column::Series(Series::new("point_pivot".into(), pivots)),
        ])?;

        println!("✅ POINT PIVOT: Result DataFrame has {} rows", result.height());
        Ok(result)
    }

    /// Groupe df par symbole et retourne HashMap<symbol, Vec<(date, open, high, low, close)>>
    fn group_by_symbol(&self, df: &DataFrame) -> Result<HashMap<String, Vec<(String, f64, f64, f64, f64)>>, PolarsError> {
        let date_col = df.column("date")?;
        let symbol_col = df.column("symbol")?;
        let open_col = df.column("open")?;
        let high_col = df.column("high")?;
        let low_col = df.column("low")?;
        let close_col = df.column("close")?;

        let mut grouped: HashMap<String, Vec<(String, f64, f64, f64, f64)>> = HashMap::new();

        for i in 0..df.height() {
            let date = date_col.get(i)?.to_string();
            let symbol = symbol_col.get(i)?.to_string();
            let open = if let AnyValue::Float64(v) = open_col.get(i)? { v } else { continue };
            let high = if let AnyValue::Float64(v) = high_col.get(i)? { v } else { continue };
            let low = if let AnyValue::Float64(v) = low_col.get(i)? { v } else { continue };
            let close = if let AnyValue::Float64(v) = close_col.get(i)? { v } else { continue };

            grouped.entry(symbol).or_insert_with(Vec::new).push((date, open, high, low, close));
        }

        Ok(grouped)
    }

    /// Calcule les pivots pour une période donnée
    /// data: toutes les données du symbole triées par date
    /// current_idx: index de la date courante
    /// period_days: nombre de jours à regarder en arrière (7, 30, 365)
    /// min_data_points: nombre minimum de points de données requis
    fn calculate_period_pivots(
        &self,
        data: &[(String, f64, f64, f64, f64)],
        current_idx: usize,
        period_days: usize,
        min_data_points: usize,
    ) -> Option<CamarillaPivot> {
        // Calculer l'index de début (au moins period_days en arrière, mais pas négatif)
        let start_idx = if current_idx >= period_days {
            current_idx - period_days + 1
        } else {
            0
        };

        // Extraire la fenêtre de données (jusqu'à current_idx inclus)
        let window = &data[start_idx..=current_idx];

        // Vérifier si on a assez de données
        if window.len() < min_data_points {
            return None;
        }

        // Calculer high.max(), low.min(), close.last(), open.first()
        let mut high_max = f64::NEG_INFINITY;
        let mut low_min = f64::INFINITY;

        for (_, _, high, low, _) in window {
            if *high > high_max {
                high_max = *high;
            }
            if *low < low_min {
                low_min = *low;
            }
        }

        let open_first = window.first()?.1; // open du premier élément
        let close_last = window.last()?.4;   // close du dernier élément

        // Calculer les pivots Camarilla
        self.calculate_camarilla_pivots(high_max, low_min, close_last, open_first)
    }

    /// Calcule les points pivots Camarilla
    fn calculate_camarilla_pivots(&self, h: f64, l: f64, c: f64, o: f64) -> Option<CamarillaPivot> {
        if h.is_nan() || l.is_nan() || c.is_nan() || o.is_nan() {
            return None;
        }

        let pivot = (h + l + c + o) / 4.0;

        Some(CamarillaPivot {
            pivot: self.round_to_2_decimals(pivot),
            r1: self.round_to_2_decimals((2.0 * pivot) - l),
            r2: self.round_to_2_decimals(pivot + (h - l)),
            r3: self.round_to_2_decimals(h + 2.0 * (pivot - l)),
            s1: self.round_to_2_decimals((2.0 * pivot) - h),
            s2: self.round_to_2_decimals(pivot - (h - l)),
            s3: self.round_to_2_decimals(l - 2.0 * (h - pivot)),
        })
    }

    fn round_to_2_decimals(&self, value: f64) -> f64 {
        (value * 100.0).round() / 100.0
    }
}