use chrono::prelude::*;
use std::io::{Error, ErrorKind};
use yahoo::time::OffsetDateTime;
use yahoo_finance_api as yahoo;

use crate::finance::operations::{max, min, n_window, price_diff};

pub async fn request_symbol_data(
    symbol: String,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Option<Vec<f64>> {
    let prices = request_closing_data(symbol.as_str(), &from, &to)
        .await
        .unwrap();

    Some(prices)
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

pub async fn request_closing_data(
    symbol: &str,
    beginning: &DateTime<Utc>,
    end: &DateTime<Utc>,
) -> std::io::Result<Vec<f64>> {
    let provider = yahoo::YahooConnector::new();

    let beginning = OffsetDateTime::from_unix_timestamp(beginning.timestamp()).unwrap();
    let end = OffsetDateTime::from_unix_timestamp(end.timestamp()).unwrap();

    let response = provider
        .get_quote_history(symbol, beginning, end)
        .await
        .map_err(|_| Error::from(ErrorKind::InvalidData))?;
    let mut quotes = response
        .quotes()
        .map_err(|_| Error::from(ErrorKind::InvalidData))?;
    if !quotes.is_empty() {
        quotes.sort_by_cached_key(|k| k.timestamp);
        Ok(quotes.iter().map(|q| q.adjclose).collect())
    } else {
        Ok(vec![])
    }
}
