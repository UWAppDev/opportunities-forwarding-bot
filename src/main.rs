/// A simple Discord forwarding bot.
/// Fetches opportunities posted to GitHub and forwards
/// them to Discord!
///
/// We use [Serenity](https://github.com/serenity-rs/serenity)
/// to create and manage the bot.
/// [This is a good tutorial on making a bot with Serenity](https://chilipepperhott.github.io/posts/intro-to-serenity/)
use std::env;

static ENV_VAR_TOKEN_NAME: &str = "DISCORD_BOT_TOKEN";

mod bot;
mod github_scraper;

fn get_bot_token() -> Option<String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // We wern't given an argument.
        // Check the environment.
        return match env::var(ENV_VAR_TOKEN_NAME) {
            Ok(s) => Some(s.clone()),
            Err(_reason) => None
        };
    }

    if args[1] == "--help" {
        print_usage(&args[0][..]);
        return None;
    }

    let token = args[1].clone();
    Some(token)
}

#[tokio::main]
async fn main() {
    match get_bot_token() {
        Some(token) => bot::start(token).await,
        None => {
            println!("Error: No API token provided.");
            std::process::exit(1);
        }
    }
}

fn print_usage(app_name: &str) {
    println!("Usage: {} <bot token>", app_name);
    println!(" If <bot token> is not provided, the contents of
the environment variable, {} are used.", ENV_VAR_TOKEN_NAME);
}

