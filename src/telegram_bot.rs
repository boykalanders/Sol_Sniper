use teloxide::{
    prelude::*,
    types::Message,
    Bot,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::profit_db::ProfitDatabase;
use tracing::{info, error};
use anyhow::Result;

pub struct TelegramController {
    bot: Bot,
    profit_db: Arc<Mutex<ProfitDatabase>>,
    authorized_users: Vec<String>,
    is_running: Arc<Mutex<bool>>,
}

impl TelegramController {
    pub fn new(
        bot_token: String,
        profit_db: ProfitDatabase,
        authorized_users: Vec<String>,
    ) -> Self {
        let bot = Bot::new(bot_token);
        let profit_db = Arc::new(Mutex::new(profit_db));
        let is_running = Arc::new(Mutex::new(true));

        Self {
            bot,
            profit_db,
            authorized_users,
            is_running,
        }
    }

    /// Check if user is authorized
    fn is_authorized(&self, user_id: &str) -> bool {
        self.authorized_users.contains(&user_id.to_string())
    }

    /// Start the Telegram bot
    pub async fn start(self) -> Result<()> {
        info!("🤖 Starting Telegram bot...");
        
        let handler = Update::filter_message().branch(
            dptree::filter(|msg: Message| {
                msg.text().is_some() && msg.from().is_some()
            })
            .endpoint(move |msg: Message| {
                let controller = self.clone();
                async move {
                    controller.handle_message(msg).await;
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                }
            }),
        );

        Dispatcher::builder(self.bot.clone(), handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }

    /// Handle incoming messages
    async fn handle_message(&self, msg: Message) {
        let text = msg.text().unwrap_or("");
        let user_id = msg.from().map(|user| user.id.to_string()).unwrap_or_default();
        let username = msg.from().and_then(|user| user.username.clone()).unwrap_or_default();

        info!("📱 Received message from {} ({}): {}", username, user_id, text);

        // Check authorization
        if !self.is_authorized(&user_id) {
            let response = "❌ Unauthorized access. You are not authorized to use this bot.";
            if let Err(e) = self.bot.send_message(msg.chat.id, response).await {
                error!("Failed to send unauthorized message: {}", e);
            }
            return;
        }

        // Handle commands
        match text.to_lowercase().trim() {
            "/start" | "/help" => {
                self.send_help_message(msg.chat.id).await;
            }
            "/profit" | "profit" => {
                self.send_profit_info(msg.chat.id).await;
            }
            "/reset" | "reset" => {
                self.reset_profit(msg.chat.id).await;
            }
            "/stop" | "stop" => {
                self.stop_bot(msg.chat.id).await;
            }
            "/start_bot" | "start_bot" => {
                self.start_bot(msg.chat.id).await;
            }
            "/status" | "status" => {
                self.send_status(msg.chat.id).await;
            }
            _ => {
                let response = "❓ Unknown command. Use /help to see available commands.";
                if let Err(e) = self.bot.send_message(msg.chat.id, response).await {
                    error!("Failed to send unknown command message: {}", e);
                }
            }
        }
    }

    /// Send help message
    async fn send_help_message(&self, chat_id: ChatId) {
        let help_text = r#"
🤖 **Sniper Bot Control Panel**

**Available Commands:**
• `/profit` - Show current profit statistics
• `/reset` - Reset all profit data to zero
• `/stop` - Stop the bot (pause trading)
• `/start_bot` - Start the bot (resume trading)
• `/status` - Show bot status
• `/help` - Show this help message

**Usage:**
Send any of these commands to control the bot.
Only authorized users can use these commands.
        "#;

        if let Err(e) = self.bot.send_message(chat_id, help_text).await {
            error!("Failed to send help message: {}", e);
        }
    }

    /// Send profit information
    async fn send_profit_info(&self, chat_id: ChatId) {
        let profit_db = self.profit_db.lock().await;
        
        match profit_db.get_profit_summary() {
            Ok(summary) => {
                let response = format!("📊 **Profit Statistics**\n\n{}", summary);
                if let Err(e) = self.bot.send_message(chat_id, response).await {
                    error!("Failed to send profit info: {}", e);
                }
            }
            Err(e) => {
                let response = format!("❌ Error getting profit data: {}", e);
                if let Err(e) = self.bot.send_message(chat_id, response).await {
                    error!("Failed to send error message: {}", e);
                }
            }
        }
    }

    /// Reset profit data
    async fn reset_profit(&self, chat_id: ChatId) {
        let profit_db = self.profit_db.lock().await;
        
        match profit_db.reset_profit() {
            Ok(_) => {
                let response = "✅ **Profit data reset successfully!**\n\nAll profit statistics have been reset to zero.";
                if let Err(e) = self.bot.send_message(chat_id, response).await {
                    error!("Failed to send reset confirmation: {}", e);
                }
                info!("💰 Profit data reset via Telegram command");
            }
            Err(e) => {
                let response = format!("❌ Error resetting profit data: {}", e);
                if let Err(e) = self.bot.send_message(chat_id, response).await {
                    error!("Failed to send reset error: {}", e);
                }
            }
        }
    }

    /// Stop the bot
    async fn stop_bot(&self, chat_id: ChatId) {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
        
        let response = "🛑 **Bot stopped successfully!**\n\nThe bot has been paused and will not execute new trades.\nUse `/start_bot` to resume trading.";
        if let Err(e) = self.bot.send_message(chat_id, response).await {
            error!("Failed to send stop confirmation: {}", e);
        }
        
        info!("🛑 Bot stopped via Telegram command");
    }

    /// Start the bot
    async fn start_bot(&self, chat_id: ChatId) {
        let mut is_running = self.is_running.lock().await;
        *is_running = true;
        
        let response = "✅ **Bot started successfully!**\n\nThe bot is now active and will execute trades based on signals.";
        if let Err(e) = self.bot.send_message(chat_id, response).await {
            error!("Failed to send start confirmation: {}", e);
        }
        
        info!("✅ Bot started via Telegram command");
    }

    /// Send bot status
    async fn send_status(&self, chat_id: ChatId) {
        let is_running = self.is_running.lock().await;
        let profit_db = self.profit_db.lock().await;
        
        let status = if *is_running { "🟢 **ACTIVE**" } else { "🔴 **STOPPED**" };
        
        match profit_db.get_profit() {
            Ok(stats) => {
                let response = format!(
                    "📊 **Bot Status**\n\n\
                    Status: {}\n\
                    Total Profit: {:.4} SOL\n\
                    Total Trades: {}\n\
                    Win Rate: {:.1}%\n\
                    Last Updated: {}",
                    status,
                    stats.total_profit,
                    stats.total_trades,
                    stats.win_rate(),
                    stats.updated_at
                );
                
                if let Err(e) = self.bot.send_message(chat_id, response).await {
                    error!("Failed to send status: {}", e);
                }
            }
            Err(e) => {
                let response = format!("❌ Error getting status: {}", e);
                if let Err(e) = self.bot.send_message(chat_id, response).await {
                    error!("Failed to send status error: {}", e);
                }
            }
        }
    }

    /// Check if bot is running
    pub async fn is_bot_running(&self) -> bool {
        let is_running = self.is_running.lock().await;
        *is_running
    }

    /// Get profit database reference
    pub fn get_profit_db(&self) -> Arc<Mutex<ProfitDatabase>> {
        self.profit_db.clone()
    }
}

impl Clone for TelegramController {
    fn clone(&self) -> Self {
        Self {
            bot: self.bot.clone(),
            profit_db: self.profit_db.clone(),
            authorized_users: self.authorized_users.clone(),
            is_running: self.is_running.clone(),
        }
    }
}

/// Send notification to Telegram
pub async fn send_telegram_notification(
    bot_token: &str,
    chat_id: &str,
    message: &str,
) -> Result<()> {
    let bot = Bot::new(bot_token.to_string());
    
    // Parse chat_id - handle both @username and numeric IDs
    let chat_id = if chat_id.starts_with('@') {
        // For @username format, we need to get the chat ID first
        // This is a simplified approach - in practice you might want to store numeric IDs
        chat_id.to_string()
    } else {
        // Assume it's already a numeric ID
        chat_id.to_string()
    };

    match bot.send_message(chat_id, message).await {
        Ok(_) => {
            info!("✅ Telegram notification sent successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to send Telegram notification: {}", e);
            Err(anyhow::anyhow!("Telegram send failed: {}", e))
        }
    }
} 