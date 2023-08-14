use crate::actors::fetcher::{Fetcher, StartFechting};
use crate::cmd::cli::{Cli, Fetch, SubCommands};
use actix::prelude::*;
use actors::calculator::{CalculateMessage, Calculator};
use clap::Parser;
use futures::future::try_join_all;

mod actors;
mod cmd;
mod finance;

#[actix::main]
async fn main() {
    let cli = Cli::parse();

    match cli.sub_commands {
        Some(SubCommands::Fetch(fetch)) => fetch_from_symbol(fetch).await,
        Some(SubCommands::FetchFromFile(fetch)) => panic!("no commands"),
        None => panic!("no valid commands!"),
    }
}

async fn fetch_from_symbol(fetch: Fetch) {
    let symbols = fetch
        .symbols
        .split(',')
        .map(|i| i.into())
        .collect::<Vec<String>>();

    let arb = Arbiter::new();
    let futs = symbols.into_iter().map(|symbol| {
        Supervisor::start_in_arbiter(&arb.handle(), |_ctx| Fetcher).send(StartFechting {
            symbol,
            from: fetch.from,
        })
    });

    let request_symbol_data_responses = try_join_all(futs).await.unwrap();
    println!("{:?}", request_symbol_data_responses);

    let futs = request_symbol_data_responses
        .into_iter()
        .map(|request_symbol_data_response| {
            Supervisor::start_in_arbiter(&arb.handle(), |_ctx| Calculator).send(CalculateMessage {
                stock_data: request_symbol_data_response.prices,
                symbol: request_symbol_data_response.symbol,
                from: request_symbol_data_response.from,
            })
        });

    let calculation_responses = try_join_all(futs).await.unwrap();
    println!("{:?}", calculation_responses);
}
