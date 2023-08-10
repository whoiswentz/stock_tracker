use chrono::prelude::*;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version)]
#[command(
    about = "stock-tracer - Track stocks and give some metrics",
    long_about = "stock-tracer - Track stocks and give some metrics"
)]
pub struct Cli {
    #[command(subcommand)]
    pub sub_commands: Option<SubCommands>,
}

#[derive(Subcommand)]
pub enum SubCommands {
    Fetch(Fetch),
    FetchFromFile(FetchFromFile),
}

#[derive(Args, Clone)]
pub struct Fetch {
    #[arg(short = 'f', long = "from")]
    pub from: DateTime<Utc>,
    #[arg(short = 's', long = "symbols")]
    pub symbols: String,
    #[arg(short = 'd', long = "duration")]
    pub duration: u64,
}

#[derive(Args, Clone)]
pub struct FetchFromFile {
    #[arg(short = 'f', long = "from")]
    pub from: DateTime<Utc>,
    #[arg(short = 'p', long = "path")]
    pub path: String,
    #[arg(short = 'd', long = "duration")]
    pub duration: u64,
}
