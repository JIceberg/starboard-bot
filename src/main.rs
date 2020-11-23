use dotenv;
use serenity::{async_trait, model::{channel::*, gateway::Ready}, prelude::*};
// use std::convert::TryFrom;

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
    dotenv::dotenv().expect("Could not read .env file");
    let token = dotenv::var("DISCORD_TOKEN")
        .expect("Could not read environment variable");
        
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}