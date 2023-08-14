use chrono::{DateTime, Utc};

pub fn min(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MAX, |acc, n| acc.min(*n)))
    }
}

pub fn max(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MIN, |acc, n| acc.max(*n)))
    }
}

pub fn price_diff(series: &[f64]) -> Option<(f64, f64)> {
    if !series.is_empty() {
        let (first, last) = (series.first().unwrap(), series.last().unwrap());
        let abs_diff = last - first;
        let first = if *first == 0.0 { 1.0 } else { *first };
        let rel_diff = abs_diff / first;
        Some((abs_diff, rel_diff))
    } else {
        None
    }
}

pub fn n_window(window_size: usize, series: &[f64]) -> Option<Vec<f64>> {
    if !series.is_empty() && window_size > 1 {
        Some(
            series
                .windows(window_size)
                .map(|w| w.iter().sum::<f64>() / w.len() as f64)
                .collect(),
        )
    } else {
        None
    }
}

pub async fn calculate(symbol: String, from: DateTime<Utc>, prices: Vec<f64>) -> String {
    let last_price = prices.last().unwrap();
    let (_, rel_diff) = price_diff(&prices).unwrap();
    let period_min = min(&prices).unwrap();
    let period_max = max(&prices).unwrap();
    let windows = n_window(30, &prices).unwrap();

    [
        symbol,
        from.to_rfc3339(),
        last_price.to_string(),
        (rel_diff * 100.00).to_string(),
        period_min.to_string(),
        period_max.to_string(),
        windows.last().unwrap_or(&0.0).to_string(),
    ]
    .join(",")
}
