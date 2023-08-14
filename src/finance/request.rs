use chrono::prelude::*;
use std::io::{Error, ErrorKind};
use yahoo::time::OffsetDateTime;
use yahoo_finance_api as yahoo;

#[derive(Debug)]
pub struct RequestSymbolDataResponse {
    pub symbol: String,
    pub prices: Vec<f64>,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

pub async fn request_symbol_data(
    symbol: String,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Option<RequestSymbolDataResponse> {
    let prices = request_closing_data(symbol.as_str(), &from, &to)
        .await
        .unwrap();

    Some(RequestSymbolDataResponse {
        symbol,
        prices,
        from,
        to,
    })
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
