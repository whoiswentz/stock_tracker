use crate::finance::request::request_symbol_data;
use actix::{
    Actor, ActorFutureExt, AsyncContext, Context, Handler, Message, ResponseActFuture, WrapFuture,
};
use chrono::prelude::*;
use std::time::Duration;

pub struct Fetcher {
    run_duration: u64,
}

#[derive(Message)]
#[rtype(result = "Vec<f64>")]
pub struct StartFechting {
    pub symbol: String,
    pub from: DateTime<Utc>,
}

impl Actor for Fetcher {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(self.run_duration), |act, ctx| {});
    }
}

impl Handler<StartFechting> for Fetcher {
    type Result = ResponseActFuture<Self, Vec<f64>>;

    fn handle(&mut self, msg: StartFechting, _ctx: &mut Context<Self>) -> Self::Result {
        let to = Utc::now();

        Box::pin(
            async move {
                let result = request_symbol_data(msg.symbol, msg.from, to).await;
                result.unwrap()
            }
            .into_actor(self)
            .map(|res, _act, _ctx| res),
        )
    }
}
