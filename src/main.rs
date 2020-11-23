use serenity::{async_trait, model::{channel::*, gateway::Ready}, prelude::*};
use std::convert::TryFrom;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
        if msg.content == "<:pogsire:727216449432846476>" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pog!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let mut client = Client::builder(&"NzgwMTgzNzkwNDI0ODE3Njc1.X7rYxg.mFCv0y5Roih31VcyMZnFflgicj0")
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}