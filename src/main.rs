use chrono::prelude::*;
use clap::{Args, Parser, Subcommand};
use std::io::{Error, ErrorKind};
use tokio_test;
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
        let stocks = symbols
            .iter()
            .map(|symbol| fetch_closing_data(symbol, &fetch.from, &fetch.to));
        for stock in stocks {
            match stock {
                Ok(history_prices) => println!("{:?}", history_prices),
                Err(error) => println!("{}", error),
            }
        }
    }
}

fn split_symbols(symbols: &str) -> Vec<&str> {
    symbols.split(',').collect::<Vec<&str>>()
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
