use chrono::prelude::*;
use clap::{Args, Parser, Subcommand};
use std::io::{Error, ErrorKind};
use yahoo::time::OffsetDateTime;
use yahoo_finance_api as yahoo;

#[derive(Parser)]
#[command(author, version)]
#[command(
    about = "stock-tracer - Track stocks and give some metrics",
    long_about = "stock-tracer - Track stocks and give some metrics"
)]

struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Fetch(Fetch),
}

#[derive(Args)]
struct Fetch {
    #[arg(short = 'f', long = "from")]
    from: DateTime<Utc>,
    #[arg(short = 't', long = "to")]
    to: DateTime<Utc>,
    #[arg(short = 's', long = "symbols")]
    symbols: String,
}

fn main() {
    let cli = Cli::parse();

    if let Some(Commands::Fetch(fetch)) = cli.command {
        let symbols = split_symbols(&fetch.symbols);
        let _ = symbols
            .iter()
            .map(|&symbol| handle_symbol_data(symbol, &fetch.from, &fetch.to))
            .collect::<Vec<_>>();
    }
}

fn split_symbols(symbols: &str) -> Vec<&str> {
    symbols.split(',').collect::<Vec<&str>>()
}

fn handle_symbol_data(symbol: &str, from: &DateTime<Utc>, to: &DateTime<Utc>) -> Option<Vec<f64>> {
    let prices = fetch_closing_data(symbol, from, to).unwrap();

    let last_price = prices.last().unwrap();
    let (_, rel_diff) = price_diff(&prices).unwrap();
    let period_min = min(&prices).unwrap();
    let period_max = max(&prices).unwrap();
    let windows = n_window(30, &prices).unwrap();

    println!(
        "{} - {}, {}, {}, {}, {}, {}",
        from.to_rfc3339(),
        symbol,
        last_price,
        rel_diff * 100.00,
        period_min,
        period_max,
        windows.last().unwrap_or(&0.0)
    );

    Some(prices)
}

fn fetch_closing_data(
    symbol: &str,
    beginning: &DateTime<Utc>,
    end: &DateTime<Utc>,
) -> std::io::Result<Vec<f64>> {
    let provider = yahoo::YahooConnector::new();

    let beginning = OffsetDateTime::from_unix_timestamp(beginning.timestamp()).unwrap();
    let end = OffsetDateTime::from_unix_timestamp(end.timestamp()).unwrap();

    let response = tokio_test::block_on(provider.get_quote_history(symbol, beginning, end))
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

fn min(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MAX, |acc, n| acc.min(*n)))
    }
}

fn max(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MIN, |acc, n| acc.max(*n)))
    }
}

fn price_diff(series: &[f64]) -> Option<(f64, f64)> {
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

fn n_window(window_size: usize, series: &[f64]) -> Option<Vec<f64>> {
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
