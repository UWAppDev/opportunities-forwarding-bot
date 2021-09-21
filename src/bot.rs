use serenity::{
    async_trait,
    model::{ channel::Message, gateway::Ready, id::ChannelId, client::Context },
    prelude::*
};

use crate::github_scraper;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        let cache = &context.cache;

        if msg.channel_id.name(cache) == "opportunities" {
            println!("Message posted in the opportunities channel! {}", msg.content);

            let dm = msg
                .author
                .dm(&context, |m| {
                    m.content(format!("Please post opportunities here: {}", github_scraper::SOURCE_URL));
                    m
                }).await;

            let deletion = msg.delete(cache).await;

            if let Err(r) = dm {
                println!("Error: {:?}", r);
            }

            if let Err(r) = deletion {
                println!("Error deleting post! {:?}", r);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // Find the channel we want to post to.
    }
}

/// Starts the forwarding bot.
/// [token] should be gotten from Discord and will allow
/// us to communicate with the Discord API.
pub async fn start(token: String) {
    // Connect to Discord!
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .await
        .expect("Unable to connect to Discord!");

    client.start().await.expect("Bot stopped!");
}

