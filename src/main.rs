#[allow(dead_code)]
mod actions;
mod api;

use dotenv::dotenv;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!create" {
            if let Err(why) = msg
                .channel_id
                .say(
                    &ctx.http,
                    "Creating server...\nThis process might take a few minutes.",
                )
                .await
            {
                println!("Error sending message: {:?}", why);
            }
            actions::spawn_new_server(&msg, &ctx).await;
        }

        if msg.content == "!destroy" {
            if let Err(why) = msg
                .reply(
                    &ctx.http,
                    "Destroying server...\nThis process might take a few minutes.",
                )
                .await
            {
                println!("Error sending message: {:?}", why);
            }
            actions::kill_server(&msg, &ctx).await
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = dotenv::var("BOT_TOKEN").expect("BOT_TOKEN not set.");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
