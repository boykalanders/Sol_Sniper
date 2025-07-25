use anyhow::{Context, Result};
use regex::Regex;
use serenity::{
    async_trait,
    client::{Client, Context as SerenityContext, EventHandler},
    gateway::ActivityData,
    model::{
        channel::Message,
        gateway::Ready,
        id::ChannelId,
    },
    prelude::*,
};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tracing::{error, info, warn};

pub struct DiscordHandler {
    pub config: crate::Config,
    pub payer: Pubkey,
    pub target_channel_id: ChannelId,
}

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn ready(&self, ctx: SerenityContext, ready: Ready) {
        info!("Discord bot {} is connected!", ready.user.name);
        
        // Set bot activity status
        let activity = ActivityData::watching("for trading signals");
        ctx.set_activity(Some(activity));
    }

    async fn message(&self, _ctx: SerenityContext, msg: Message) {
        // Only process messages from the target channel
        if msg.channel_id != self.target_channel_id {
            return;
        }

        // Skip bot messages
        if msg.author.bot {
            return;
        }

        // Process the message for trading signals
        if let Some(signal) = self.parse_trading_signal(&msg.content).await {
            info!("Trading signal detected: {:?}", signal);
            
            // Trigger buy action
            let config = self.config.clone();
            let payer = self.payer;
            tokio::spawn(crate::buy::execute(signal.token_address, config, payer));
            
            // Send notification
            let notification = format!(
                "🚀 Signal detected!\nToken: {}\nSignal: {}\nChannel: {}",
                signal.token_address,
                signal.signal_type,
                msg.channel_id
            );
            crate::notifier::log(notification).await;
        }
    }
}

#[derive(Debug, Clone)]
pub struct TradingSignal {
    pub token_address: Pubkey,
    pub signal_type: String,
    pub confidence: f32,
}

impl DiscordHandler {
    async fn parse_trading_signal(&self, content: &str) -> Option<TradingSignal> {
        let content = content.to_lowercase();
        
        // Check for signal keywords
        let signal_patterns = [
            r"(?i)\b(buy|long|entry|signal|pump|rocket|moon)\b",
            r"(?i)\b(🚀|📈|💎|🔥|⚡)\b",
            r"(?i)\b(new\s+token|gem|pick)\b",
            r"(?i)\b(CA)\b",
            r"(?i)\b(hello)\b",
        ];
        
        let has_signal = signal_patterns.iter().any(|pattern| {
            Regex::new(pattern).unwrap().is_match(&content)
        });
        
        if !has_signal {
            return None;
        }

        // Extract Solana token address (base58 format, typically 32-44 characters)
        let token_patterns = [
            // Standard Solana address pattern
            r"([A-Za-z0-9]{32,44})\b",
            // Contract address with common prefixes
            r"(?i)(?:address|contract|token|ca)[:=\s]+([A-Za-z0-9]{32,44})",
            // Token address after common terms
            r"(?i)(?:token|contract|address)\s*[:=]?\s*([A-Za-z0-9]{32,44})",
        ];

        for pattern in &token_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for cap in re.captures_iter(&content) {
                    if let Some(addr_match) = cap.get(1) {
                        let addr_str = addr_match.as_str();
                        
                        // Validate it's a proper Solana address
                        if let Ok(pubkey) = Pubkey::from_str(addr_str) {
                            // Additional validation - check if it looks like a token mint
                            if self.is_likely_token_address(&pubkey).await {
                                let signal_type = self.detect_signal_type(&content);
                                let confidence = self.calculate_confidence(&content);
                                
                                return Some(TradingSignal {
                                    token_address: pubkey,
                                    signal_type,
                                    confidence,
                                });
                            }
                        }
                    }
                }
            }
        }

        warn!("Signal detected but no valid token address found in: {}", content);
        None
    }

    async fn is_likely_token_address(&self, _pubkey: &Pubkey) -> bool {
        // For now, assume all valid pubkeys are potential tokens
        // In production, you might want to validate against known token programs
        // or check if the account exists on-chain
        true
    }

    fn detect_signal_type(&self, content: &str) -> String {
        let content = content.to_lowercase();
        
        if content.contains("buy") || content.contains("long") || content.contains("entry") {
            "BUY".to_string()
        } else if content.contains("pump") || content.contains("rocket") || content.contains("🚀") {
            "PUMP".to_string()
        } else if content.contains("gem") || content.contains("💎") {
            "GEM".to_string()
        } else {
            "SIGNAL".to_string()
        }
    }

    fn calculate_confidence(&self, content: &str) -> f32 {
        let content = content.to_lowercase();
        let mut confidence: f32 = 0.5; // Base confidence
        
        // Boost confidence for multiple signal indicators
        if content.contains("🚀") { confidence += 0.1; }
        if content.contains("buy") { confidence += 0.2; }
        if content.contains("gem") { confidence += 0.15; }
        if content.contains("new") { confidence += 0.1; }
        if content.contains("token") { confidence += 0.05; }
        
        // Cap at 1.0
        confidence.min(1.0)
    }
}

pub async fn run(config: crate::Config, payer: Pubkey) -> Result<()> {
    let token = &config.discord_bot_token;
    let channel_id = config.discord_channel_id.parse::<u64>()
        .context("Invalid Discord channel ID")?;

    let intents = GatewayIntents::GUILD_MESSAGES 
        | GatewayIntents::MESSAGE_CONTENT;

    let handler = DiscordHandler {
        config: config.clone(),
        payer,
        target_channel_id: ChannelId::new(channel_id),
    };

    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .await
        .context("Failed to create Discord client")?;

    info!("Starting Discord bot to monitor channel ID: {}", channel_id);

    if let Err(why) = client.start().await {
        error!("Discord client error: {:?}", why);
        return Err(anyhow::anyhow!("Discord client failed: {}", why));
    }

    Ok(())
} 