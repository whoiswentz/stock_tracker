use crate::finance::operations::calculate;
use actix::{
    Actor, ActorFutureExt, Context, Handler, Message, ResponseActFuture, Supervised, WrapFuture,
};
use chrono::prelude::*;

pub struct Calculator;

#[derive(Message)]
#[rtype(result = "String")]
pub struct CalculateMessage {
    pub stock_data: Vec<f64>,
    pub symbol: String,
    pub from: DateTime<Utc>,
}

impl Actor for Calculator {
    type Context = Context<Self>;
}

impl Handler<CalculateMessage> for Calculator {
    type Result = ResponseActFuture<Self, String>;

    fn handle(&mut self, msg: CalculateMessage, _ctx: &mut Context<Self>) -> Self::Result {
        Box::pin(
            async move { calculate(msg.symbol, msg.from, msg.stock_data).await }
                .into_actor(self)
                .map(|res, _act, _ctx| res),
        )
    }
}

impl Supervised for Calculator {
    fn restarting(&mut self, ctx: &mut Context<Self>) {
        println!("restarting");
    }
}
