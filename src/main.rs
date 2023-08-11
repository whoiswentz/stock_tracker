use crate::actors::fetcher::{Fetcher, StartFechting};
use crate::cmd::cli::{Cli, Fetch, SubCommands};
use actix::prelude::*;
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
    let futs = fetch
        .symbols
        .split(',')
        .map(|i| i.into())
        .collect::<Vec<String>>()
        .into_iter()
        .map(|symbol| (symbol, Arbiter::new()))
        .map(|(symbol, arb)| {
            (
                symbol,
                Fetcher::start_in_arbiter(&arb.handle(), |_ctx| Fetcher),
            )
        })
        .map(|(symbol, addr)| {
            addr.send(StartFechting {
                symbol,
                from: fetch.from,
            })
        });

    let res = try_join_all(futs).await;

    println!("{:?}", res)
}
