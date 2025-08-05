use teloxide::{
    prelude::*,
    types::Message,
    Bot,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Get bot token from environment or config
    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
        .unwrap_or_else(|_| {
            println!("Please set TELEGRAM_BOT_TOKEN environment variable");
            println!("Or modify this script to read from config.toml");
            std::process::exit(1);
        });

    let bot = Bot::new(bot_token);
    
    println!("🤖 Starting Telegram ID helper bot...");
    println!("Send any message to this bot to get your user ID");
    println!("Press Ctrl+C to stop");

    let handler = Update::filter_message().branch(
        dptree::filter(|msg: Message| {
            msg.text().is_some() && msg.from().is_some()
        })
        .endpoint(move |msg: Message| {
            let bot = bot.clone();
            async move {
                let user = msg.from().unwrap();
                let user_id = user.id;
                let username = user.username.clone().unwrap_or_else(|| "No username".to_string());
                
                let response = format!(
                    "👤 **Your Telegram Information**\n\n\
                    **User ID:** `{}`\n\
                    **Username:** @{}\n\
                    **To authorize this user:**\n\
                    Add `\"{}\"` to the `tg_authorized_users` array in your `config.toml`\n\n\
                    **Example:**\n\
                    ```toml\n\
                    tg_authorized_users = [\"{}\"]\n\
                    ```",
                    user_id, username, user_id, user_id
                );

                if let Err(e) = bot.send_message(msg.chat.id, response).await {
                    eprintln!("Failed to send message: {}", e);
                }
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            }
        }),
    );

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
} 