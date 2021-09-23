# opportunities-forwarding-bot
A Discord bot that forwards content from GitHub!

# Testing it
 * First, make sure you update it to fetch from your own GitHub repo! Currently, this must be changed by editing `src/github_scraper.rs`.
   * Without changing this, the bot will take opportunities from [this repository's discussion tab](https://github.com/UWAppDev/opportunities-forwarding-bot/discussions/categories/opportunities).
 * Next, you'll need to create a Discord application and add a bot to it.
   * [This tutorial explains how to do that.](https://discordjs.guide/preparations/setting-up-a-bot-application.html#creating-your-bot)
   * When generating your token, make sure you give the bot these permissions:
     ![Enable the `bot` and `messages.read` permissions](https://user-images.githubusercontent.com/46334387/134440907-ddb5a504-4f01-4828-ab72-9cab788c86a3.png)
     ![In `Bot Settings`, enable `Send Messages`, `Public Threads`, `Send Messages in Threads`, `Manage Messages`, `Embed Links`, `Attach Files`, `Read Message History`, `View Channels`, and `Add Reactions`](https://user-images.githubusercontent.com/46334387/134440921-61e8162e-a445-49e7-bc3e-22b74466ade3.png)
 * After setting up the application, `clone` this repository. [If you haven't installed `rust` and `cargo`, do so now.](https://www.rust-lang.org/)
 * Make sure you've invited the bot to a server with a public channel named `opportunities`!
 * After building the repository (via `cargo build`), start the bot using `cargo run "<Your token goes here>"`.
   * The bot should forward opportunities from the GitHub repository's opportunities discussion category to your Discord server!
