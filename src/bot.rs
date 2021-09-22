use serenity::{
    async_trait,
    client::{ Context },
    cache::Cache,
    http::client::Http,
    model::{ channel::Message, gateway::Ready, id::ChannelId, channel::ReactionType },
    prelude::*
};
use serenity::futures::StreamExt;

use crate::github_scraper;
use std::sync::Arc;

macro_rules! DELETED_MESSAGE_WARNING { () => { "I've deleted your message from the opportunities channel. It said: \n\n{}\n\nPlease post opportunities here: {}" }; }

struct Handler;

impl Handler {
    /// Delete an illegal message, [msg] and direct messages the author an appropriate
    /// explanation.
    /// If unable to delete the message (an error!) no direct message is sent to the author.
    async fn block_illegal_post(&self, context: Context, msg: &Message) -> Result<(), SerenityError> {
        let reply_text = format!(DELETED_MESSAGE_WARNING!(), msg.content, github_scraper::SOURCE_URL);

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

    /// Get a list of all channels we should manage.
    async fn get_target_channels(&self, ctx: Context) -> Result<Vec<ChannelId>, SerenityError> {
        let mut result: Vec<ChannelId> = Vec::new();
        let cache: Arc<Cache> = ctx.cache;
        let http: Arc<Http> = ctx.http;

        let user = http.get_current_user().await?;
        let guilds = user.guilds(http.clone()).await?;

        // Search each guild for target channels.
        let guild_ids =
            guilds.iter()
                .map(|guild| guild.id);

        for guild_id in guild_ids {
            let channels = guild_id.channels(http.clone()).await?;

            for (channel_id, _channel) in channels.iter() {
                let channel_name = channel_id.name(cache.clone()).await;

                if self.is_target_channel(&channel_name) {
                    result.push(channel_id.clone());
                }
            }
        }

        Ok(result)
    }

    /// Delete all illegal posts from [channel]. A message is considered illegal if
    /// it was posted after the bot's last post in [channel].
    async fn delete_illegal_posts(&self, context: Context, channel: &ChannelId) -> Result<(), SerenityError> {
        let mut found_own = false;
        let mut target_posts: Vec<Box<Arc<Message>>> = Vec::new();

        // See documentation for ChannelId::messages_iter.
        let mut messages_stream = channel.messages_iter(&context).boxed();
        while let Some(message) = messages_stream.next().await {
            let message = message?;

            // Stop when we encounter something we've posted.
            // We only want to delete posts made while we've been away.
            if message.is_own(&context).await {
                found_own = true;
                break;
            }

            target_posts.push(Box::new(Arc::new(message)));
        }

        if found_own {
            for message in target_posts {
                self.block_illegal_post(context.clone(), &message).await?;
            }
        }

        Ok(())
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

    async fn handle_channel(&self, context: Context, channel_id: &ChannelId) -> Result<(), SerenityError> {
        self.delete_illegal_posts(context.clone(), &channel_id).await?;
        self.forward_opportunities(context.http.clone(), &channel_id).await?;

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

        // Make sure we're not the one who posted the message.
        if msg.is_own(cache).await {
            // We can post messages in the opportunities channel.
            return;
        }

        if self.is_target_channel(&name) {
            // Delete the message & dm the author.
            println!("Message posted in the opportunities channel! Deleting and replying.");

            if let Err(why) = self.block_illegal_post(context, &msg).await {
                println!("Error blocking post! {:?}", why);
            }
        }
        else if msg.is_private() {
            // For fun :)
            let reaction = ReactionType::Unicode("❓".to_string());
            let reaction = msg.react(context.http.clone(), reaction).await;
            if let Err(why) = reaction {
                println!("Error reacting to direct message: {:?}", why);
            }
        }
    }

    /// Triggered when the bot successfully connects to the server.
    /// [ctx] and [ready] provide information about the Shard (instance of the bot in a guild)
    /// and user.
    async fn ready(&self, context: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let channels;
        match self.get_target_channels(context.clone()).await {
            Ok(c) => { channels = c; },
            Err(why) => {
                println!("Unable to fetch a list of target channels: {:?}", why);
                return;
            },
        };

        for channel_id in channels.iter() {
            let res = self.handle_channel(context.clone(), &channel_id).await;

            if let Err(why) = res {
                println!("Error forwarding opportunities to a channel: {:?}", why);
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

