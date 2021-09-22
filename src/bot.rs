use serenity::{
    async_trait,
    client::{ Context },
    cache::Cache,
    http::client::Http,
    model::{ channel::Message, gateway::Ready, id::ChannelId },
    prelude::*
};

use crate::github_scraper;
use std::sync::Arc;

macro_rules! DELETED_MESSAGE_WARNING { () => { "I've deleted your message from the opportunities channel. It said: \n\n{}\n\nPlease post opportunities here: {}" }; }

struct Handler;

impl Handler {
    /// Delete an illegal message, [msg] and direct message the author with [reply_text].
    /// If unable to delete the message (an error!) no direct message is sent to the author.
    async fn block_illegal_post(&self, reply_text: String, context: Context, msg: Message) -> Result<(), SerenityError> {
        let deletion = msg.delete(context.http.clone()).await;
        if let Err(why) = deletion {
            return Err(why);
        }

        let reply = msg.author
            .dm(&context, |m| {
                m.content(reply_text);
                m
            }).await;
        if let Err(why) = reply {
            return Err(why);
        }

        Ok(())
    }

    /// Returns whether a channel with the given name applies to this.
    fn is_target_channel(&self, channel_name: &Option<String>) -> bool {
        channel_name == &Some("opportunities".to_string())
    }

    /// Forward new opportunities posted to GitHub to [channel].
    /// Returns errors generated in creating the message.
    async fn forward_opportunities(&self, http: impl AsRef<Http>, channel: &ChannelId) -> Result<(), SerenityError> {
        let msg = channel.send_message(http, |m| {
            m.content("test");

            m
        }).await;

        if let Err(why) = msg {
            return Err(why);
        }

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    /// Handle a message posted (by a user) to the opportunities channel.
    ///
    async fn message(&self, context: Context, msg: Message) {
        let cache = &context.cache;
        let name = msg.channel_id.name(cache).await;
        let my_id;

        match context.http.get_current_user().await {
            Err(why) => {
                println!("Error getting current user ID! {:?}", why);
                return;
            },
            Ok(id) => {
                my_id = id;
            },
        };

        // Make sure we're not the one who posted the message.
        if msg.author.id == my_id.id {
            // We can post messages in the opportunities channel.
            return;
        }

        if self.is_target_channel(&name) {
            // Delete the message & dm the author.
            println!("Message posted in the opportunities channel! Deleting and replying.");

            let reply_text = format!(DELETED_MESSAGE_WARNING!(), msg.content, github_scraper::SOURCE_URL);

            if let Err(why) = self.block_illegal_post(reply_text, context, msg).await {
                println!("Error blocking post! {:?}", why);
            }
        }
    }

    /// Triggered when the bot successfully connects to the server.
    /// [ctx] and [ready] provide information about the Shard (instance of the bot in a guild)
    /// and user.
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let cache: Arc<Cache> = ctx.cache;
        let http: Arc<Http> = ctx.http;

        let guilds;
        match ready.user.guilds(http.clone()).await {
            Err(why) => {
                println!("Unable to fetch guilds the user is in: {:?}", why);
                return;
            },
            Ok(g) => { guilds = g; }
        };

        // Find the channel(s) we want to post to.
        let guild_ids =
            guilds.iter()
                .map(|guild| guild.id);

        for guild_id in guild_ids {
            let channels;
            match guild_id.channels(http.clone()).await {
                Ok(channel_list) => { channels = channel_list; },
                Err(why) => {
                    println!("Error getting channels: {:?}", why);
                    continue;
                }
            };

            for (channel_id, _channel) in channels.iter() {
                let channel_name = channel_id.name(cache.clone()).await;

                if self.is_target_channel(&channel_name) {
                    let res = self.forward_opportunities(http.clone(), channel_id).await;

                    if let Err(why) = res {
                        println!("Unable to forward opportunities to channel: {:?}", why);
                    }
                }
            }
        }
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

