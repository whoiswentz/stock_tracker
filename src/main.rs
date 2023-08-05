use chrono::prelude::*;
use clap::{Args, Parser, Subcommand};
use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Read};
use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::task::JoinSet;
use tokio::time::{self, Interval};
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
    sub_commands: Option<SubCommands>,
}

#[derive(Subcommand)]
enum SubCommands {
    Fetch(Fetch),
    FetchFromFile(FetchFromFile),
}

#[derive(Args, Clone)]
struct Fetch {
    #[arg(short = 'f', long = "from")]
    from: DateTime<Utc>,
    #[arg(short = 's', long = "symbols")]
    symbols: String,
    #[arg(short = 'd', long = "duration")]
    duration: u64,
}

#[derive(Args, Clone)]
struct FetchFromFile {
    #[arg(short = 'f', long = "from")]
    from: DateTime<Utc>,
    #[arg(short = 'p', long = "path")]
    path: String,
    #[arg(short = 'd', long = "duration")]
    duration: u64,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let mut cli_fns = CliFn::new();
    match cli.sub_commands {
        Some(SubCommands::Fetch(fetch)) => cli_fns.handle_fetch_sub_command(fetch).await,
        Some(SubCommands::FetchFromFile(fetch)) => {
            cli_fns.handle_fetch_from_file_subcommand(fetch).await
        }
        None => panic!("no valid commands!"),
    }

    Ok(())
}

struct CliFn {
    pub join_set: JoinSet<Option<Vec<f64>>>,
}

impl CliFn {
    pub fn new() -> Self {
        CliFn {
            join_set: JoinSet::new(),
        }
    }

    pub async fn handle_fetch_sub_command(&mut self, fetch: Fetch) {
        let symbols: Vec<String> = fetch.symbols.split(',').map(|i| i.into()).collect();
        let mut clock = time::interval(Duration::from_secs(fetch.duration));

        let to = Utc::now();
        loop {
            clock.tick().await;
            let _ = symbols
                .iter()
                .cloned()
                .map(|symbol| {
                    self.join_set
                        .spawn(handle_symbol_data(symbol, fetch.from, to))
                })
                .collect::<Vec<_>>();

            self.join().await;
        }
    }

    pub async fn handle_fetch_from_file_subcommand(&mut self, fetch: FetchFromFile) {
        let mut file = File::open(fetch.path).unwrap();
        let mut buffer = String::new();
        file.read_to_string(&mut buffer);

        let symbols = buffer
            .split(',')
            .map(|s| s.trim().into())
            .collect::<Vec<String>>();

        let mut clock = time::interval(Duration::from_secs(fetch.duration));

        let to = Utc::now();
        loop {
            clock.tick().await;
            let _ = symbols
                .iter()
                .cloned()
                .map(|symbol| {
                    self.join_set
                        .spawn(handle_symbol_data(symbol, fetch.from, to))
                })
                .collect::<Vec<_>>();

            self.join().await;
        }
    }

    async fn join(&mut self) {
        while let Some(stock) = self.join_set.join_next().await {
            match stock {
                Ok(_) => println!("processed"),
                Err(err) => println!("{}", err),
            }
        }
    }
}

async fn handle_symbol_data(
    symbol: String,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Option<Vec<f64>> {
    let prices = fetch_closing_data(symbol.as_str(), &from, &to)
        .await
        .unwrap();

    let last_price = prices.last().unwrap();
    let (_, rel_diff) = price_diff(&prices).await.unwrap();
    let period_min = min(&prices).await.unwrap();
    let period_max = max(&prices).await.unwrap();
    let windows = n_window(30, &prices).await.unwrap();

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

async fn fetch_closing_data(
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

async fn min(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MAX, |acc, n| acc.min(*n)))
    }
}

async fn max(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MIN, |acc, n| acc.max(*n)))
    }
}

async fn price_diff(series: &[f64]) -> Option<(f64, f64)> {
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

async fn n_window(window_size: usize, series: &[f64]) -> Option<Vec<f64>> {
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
