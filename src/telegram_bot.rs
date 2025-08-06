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
    notification_chat_id: Option<String>,
}

impl TelegramController {
    pub fn new(
        bot_token: String,
        profit_db: ProfitDatabase,
        authorized_users: Vec<String>,
        notification_chat_id: Option<String>,
    ) -> Self {
        let bot = Bot::new(bot_token);
        let profit_db = Arc::new(Mutex::new(profit_db));
        let is_running = Arc::new(Mutex::new(true));

        Self {
            bot,
            profit_db,
            authorized_users,
            is_running,
            notification_chat_id,
        }
    }

    /// Check if user is authorized
    fn is_authorized(&self, user_id: &str) -> bool {
        self.authorized_users.contains(&user_id.to_string())
    }

    /// Start the Telegram bot
    pub async fn start(mut self) -> Result<()> {
        info!("🤖 Starting Telegram bot...");
        
        let mut retry_count = 0;
        let max_retries = 5;
        
        loop {
            match self.try_start_once().await {
                Ok(_) => {
                    info!("🤖 Telegram bot started successfully");
                    break;
                }
                Err(e) => {
                    retry_count += 1;
                    error!("🤖 Telegram bot failed (attempt {}/{}): {}", retry_count, max_retries, e);
                    
                    if retry_count >= max_retries {
                        error!("🤖 Telegram bot failed after {} attempts, giving up", max_retries);
                        return Err(e);
                    }
                    
                    // Wait before retrying (exponential backoff)
                    let delay = std::time::Duration::from_secs(2_u64.pow(retry_count as u32));
                    info!("🤖 Retrying Telegram bot in {} seconds...", delay.as_secs());
                    tokio::time::sleep(delay).await;
                }
            }
        }
        
        Ok(())
    }

    /// Try to start the Telegram bot once with error handling
    async fn try_start_once(&mut self) -> Result<()> {
        let bot = self.bot.clone();
        let controller = self.clone();
        
        let handler = Update::filter_message().branch(
            dptree::filter(|msg: Message| {
                msg.text().is_some() && msg.from().is_some()
            })
            .endpoint(move |msg: Message| {
                let controller = controller.clone();
                async move {
                    controller.handle_message(msg).await;
                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                }
            }),
        );

        // Build the dispatcher with error handling
        let mut dispatcher = Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build();

        // Start the dispatcher - it will run until an error occurs
        // We'll catch the TerminatedByOtherGetUpdates error in the main loop
        dispatcher.dispatch().await;
        
        // If we reach here, the dispatcher has stopped
        // This usually means an error occurred
        Err(anyhow::anyhow!("Telegram bot dispatcher stopped unexpectedly"))
    }

    /// Handle incoming messages
    async fn handle_message(&self, msg: Message) {
        let text = msg.text().unwrap_or("");
        let user_id = msg.from().map(|user| user.id.to_string()).unwrap_or_default();
        let username = msg.from().and_then(|user| user.username.clone()).unwrap_or_default();

        info!("📱 Received message from {} ({}): {}", username, user_id, text);

        // Handle commands with selective authorization
        match text.to_lowercase().trim() {
            "/start" | "/help" => {
                self.send_help_message(msg.chat.id).await;
            }
            "/profit" | "profit" => {
                self.send_profit_info(msg.chat.id).await;
            }
            "/status" | "status" => {
                self.send_status(msg.chat.id).await;
            }
            "/reset" | "reset" => {
                // Check authorization for reset command
                if !self.is_authorized(&user_id) {
                    let response = "❌ Unauthorized access. Only authorized users can reset profit data.";
                    if let Err(e) = self.bot.send_message(msg.chat.id, response).await {
                        error!("Failed to send unauthorized message: {}", e);
                    }
                    return;
                }
                self.reset_profit(msg.chat.id).await;
            }
            "/stop" | "stop" => {
                // Check authorization for stop command
                if !self.is_authorized(&user_id) {
                    let response = "❌ Unauthorized access. Only authorized users can stop the bot.";
                    if let Err(e) = self.bot.send_message(msg.chat.id, response).await {
                        error!("Failed to send unauthorized message: {}", e);
                    }
                    return;
                }
                self.stop_bot(msg.chat.id).await;
            }
            "/start_bot" | "start_bot" => {
                // Check authorization for start_bot command
                if !self.is_authorized(&user_id) {
                    let response = "❌ Unauthorized access. Only authorized users can start the bot.";
                    if let Err(e) = self.bot.send_message(msg.chat.id, response).await {
                        error!("Failed to send unauthorized message: {}", e);
                    }
                    return;
                }
                self.start_bot(msg.chat.id).await;
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

**Public Commands (Anyone can use):**
• `/start` - Start the bot interface
• `/help` - Show this help message
• `/status` - Show bot status
• `/profit` - Show current profit statistics

**Authorized Commands (Admin only):**
• `/reset` - Reset all profit data to zero
• `/stop` - Stop the bot (pause trading)
• `/start_bot` - Start the bot (resume trading)

**Usage:**
Send any of these commands to interact with the bot.
Some commands require authorization.
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
                if let Err(e) = self.bot.send_message(chat_id, &response).await {
                    error!("Failed to send profit info: {}", e);
                }
                
                // Send notification
                self.send_notification(&response).await;
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
                
                // Send notification
                self.send_notification("🔄 Profit data reset executed").await;
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
        
        // Send notification
        self.send_notification("🛑 Bot stopped via Telegram command").await;
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
        
        // Send notification
        self.send_notification("✅ Bot started via Telegram command").await;
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
                
                // Send notification
                self.send_notification("📊 Bot status requested").await;
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
    #[allow(dead_code)]
    pub async fn is_bot_running(&self) -> bool {
        let is_running = self.is_running.lock().await;
        *is_running
    }

    /// Get profit database reference
    #[allow(dead_code)]
    pub fn get_profit_db(&self) -> Arc<Mutex<ProfitDatabase>> {
        self.profit_db.clone()
    }

    /// Send notification to configured chat
    async fn send_notification(&self, message: &str) {
        if let Some(chat_id) = &self.notification_chat_id {
            if let Err(e) = self.bot.send_message(chat_id.to_string(), message).await {
                error!("Failed to send notification: {}", e);
            } else {
                info!("📢 Notification sent: {}", message);
            }
        }
    }
}

impl Clone for TelegramController {
    fn clone(&self) -> Self {
        Self {
            bot: self.bot.clone(),
            profit_db: self.profit_db.clone(),
            authorized_users: self.authorized_users.clone(),
            is_running: self.is_running.clone(),
            notification_chat_id: self.notification_chat_id.clone(),
        }
    }
}

/// Send notification to Telegram
#[allow(dead_code)]
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