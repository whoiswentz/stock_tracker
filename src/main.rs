use crate::cmd::cli::{Cli, SubCommands};
use actix::prelude::*;
use actix::System;
use chrono::prelude::*;
use clap::{Args, Parser, Subcommand};
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time;
use yahoo::time::OffsetDateTime;
use yahoo_finance_api as yahoo;

mod actors;
mod cmd;
mod finance;

#[actix::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    match cli.sub_commands {
        Some(SubCommands::Fetch(fetch)) => panic!("no commands"),
        Some(SubCommands::FetchFromFile(fetch)) => panic!("no commands"),
        None => panic!("no valid commands!"),
    }
}
