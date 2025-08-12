use anyhow::{anyhow, Context, Result};
use regex::Regex;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tracing::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio::time::{interval, Interval};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use std::time::Duration;
use solana_sdk::signer::keypair::Keypair;
use crate::{Config, get_sol_balance};

pub async fn run(config: Config, payer: Arc<Keypair>, connected: Arc<AtomicBool>) -> Result<()> {
    loop {
        match connect_and_listen(&config, payer.clone(), &connected).await {
            Ok(_) => break,
            Err(e) => {
                error!("Discord connection error: {}. Reconnecting in 5s...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
    Ok(())
}

async fn connect_and_listen(config: &Config, payer: Arc<Keypair>, connected: &Arc<AtomicBool>) -> Result<()> {
    let (ws_stream, _) = connect_async("wss://gateway.discord.gg/?v=10&encoding=json")
        .await
        .context("Failed to connect to Discord Gateway")?;
    let (mut write, mut read) = ws_stream.split();
    let token = config.discord_token.clone();
    let channel_ids: Vec<String> = config.discord_channel_id.clone();
    // Check if it's a bot token (starts with "Bot ") or user token
    let is_bot_token = token.starts_with("Bot ");
    
    let identify = if is_bot_token {
        json!({
            "op": 2,
            "d": {
                "token": token,
                "intents": 33280, // GUILD_MESSAGES (512) + MESSAGE_CONTENT (32768) = 33280
                "properties": {
                    "$os": "linux",
                    "$browser": "custom",
                    "$device": "custom"
                }
            }
        })
    } else {
        // User token - different format, no intents needed
        json!({
            "op": 2,
            "d": {
                "token": token,
                "properties": {
                    "$os": "Windows",
                    "$browser": "Chrome",
                    "$device": "Desktop"
                }
            }
        })
    };
    
    info!("Connecting to Discord as {} token...", if is_bot_token { "bot" } else { "user" });
    tracing::debug!("Sending identify payload: {}", identify.to_string());
    write.send(Message::Text(identify.to_string())).await?;
    let (heartbeat_tx, mut heartbeat_rx) = mpsc::channel(1);
    tokio::spawn(async move {
        while let Some(interval_ms) = heartbeat_rx.recv().await {
            let mut interval: Interval = interval(Duration::from_millis(interval_ms));
            loop {
                interval.tick().await;
                if let Err(e) = write
                    .send(Message::Text(json!({"op": 1, "d": null}).to_string()))
                    .await
                {
                    error!("Heartbeat failed: {}", e);
                    break;
                }
            }
        }
    });
    while let Some(msg) = read.next().await {
        let msg = msg.context("WebSocket read error")?;
        if let Message::Text(text) = msg {
            tracing::debug!("Raw Discord event: {}", text);
            let event: Value = serde_json::from_str(&text)?;
            let op_code = event["op"].as_i64();
            tracing::debug!("Discord event op code: {:?}, type: {:?}", op_code, event["t"].as_str());
            match op_code {
                Some(10) => {
                    let heartbeat_interval = event["d"]["heartbeat_interval"]
                        .as_u64()
                        .unwrap_or(45000) as u64;
                    heartbeat_tx.send(heartbeat_interval).await?;
                    info!("Discord Gateway HELLO received, heartbeat interval: {}ms", heartbeat_interval);
                    connected.store(true, Ordering::Relaxed);
                    crate::notifier::log("✅ Discord Gateway connected".to_string()).await;
                }
                Some(0) => {
                    let event_type = event["t"].as_str().unwrap_or("");
                    if event_type == "READY" {
                        let user_info = &event["d"]["user"];
                        let username = user_info["username"].as_str().unwrap_or("Unknown");
                        let user_id = user_info["id"].as_str().unwrap_or("Unknown");
                        info!("🎯 Discord Gateway READY - Logged in as: {} ({})", username, user_id);
                        info!("🎯 Target channels to monitor: {:?}", channel_ids);
                    } else if event_type == "MESSAGE_CREATE" {
                        let message = &event["d"];
                        let channel_id = message["channel_id"].as_str().unwrap_or("");
                        let author_name = message["author"]["username"].as_str().unwrap_or("Unknown");
                        let content = message["content"].as_str().unwrap_or("");
                        
                        // Only log messages from target channels
                        if !channel_ids.contains(&channel_id.to_string()) {
                            continue; // Silently ignore non-target channels
                        }
                        
                        let is_bot = message["author"]["bot"].as_bool().unwrap_or(false);
                        let author_type = if is_bot { "🤖 Bot" } else { "👤 User" };
                        
                        info!("📨 Message from target channel {}: {} ({}) - '{}'", channel_id, author_name, author_type, content);
                        
                        // Forward ALL messages from target channels to Telegram (including bots)
                        let forward_message = format!(
                            "📨 Discord Message\nFrom: {} ({})\nChannel: {}\nContent: {}",
                            author_name, author_type, channel_id, content
                        );
                        crate::notifier::log(forward_message).await;
                        
                        // Check for trading signals
                        if let Some(token_address) = parse_trading_signal(content).await {
                            info!("🎯 SIGNAL DETECTED! Token: {} | From: {} | Channel: {}", token_address, author_name, channel_id);
                            info!("📝 Message content: '{}'", content);
                            
                            let config_clone = config.clone();
                            let payer_clone = payer.clone();
                            tokio::spawn(crate::buy::execute(
                                token_address,
                                config_clone,
                                payer_clone,
                            ));
                            let signal_notification = format!(
                                "🚀 SIGNAL DETECTED!\nToken: {}\nFrom: {}\nChannel: {}\nMessage: {}",
                                token_address, author_name, channel_id, content
                            );
                            crate::notifier::log(signal_notification).await;
                        }
                    }
                }
                Some(9) => {
                    // Invalid session - need to reconnect
                    error!("❌ Discord: Invalid session - token may be expired or invalid");
                    return Err(anyhow!("Invalid Discord session"));
                }
                Some(7) => {
                    // Reconnect
                    info!("🔄 Discord: Reconnecting...");
                    return Err(anyhow!("Discord reconnect requested"));
                }
                Some(4) => {
                    // Authentication failed
                    let error_code = event["d"].as_i64().unwrap_or(0);
                    error!("❌ Discord Authentication failed with code: {}", error_code);
                    match error_code {
                        4004 => error!("Authentication failed: Invalid token"),
                        4011 => error!("Authentication failed: Disallowed intents"),
                        4013 => error!("Authentication failed: Invalid intents"),
                        4014 => error!("Authentication failed: Disallowed intents (privileged)"),
                        _ => error!("Authentication failed: Unknown error {}", error_code),
                    }
                    return Err(anyhow!("Discord authentication failed: {}", error_code));
                }
                _ => {
                    tracing::debug!("Unhandled Discord op code: {:?}", op_code);
                }
            }
        }
    }
    info!("Discord WebSocket loop ended");
    Err(anyhow!("WebSocket disconnected"))
}

async fn parse_trading_signal(content: &str) -> Option<Pubkey> {
    // First, check if message contains "CA" (Contract Address)
    let signal_patterns = [r"(?i)\b(CA)\b"];
    let has_signal = signal_patterns.iter().any(|pattern| {
        Regex::new(pattern).unwrap().is_match(content)
    });
    
    if !has_signal {
        return None;
    }
    
    info!("🔍 Signal detected in message: '{}'", content);
    
    // Improved token patterns to match various formats
    let token_patterns = [
        // Pattern 1: CA: <address> (your format)
        r"(?i)ca\s*:\s*([A-Za-z0-9]{32,44})",
        // Pattern 2: CA=<address>
        r"(?i)ca\s*=\s*([A-Za-z0-9]{32,44})",
        // Pattern 3: Just the address after CA
        r"(?i)ca\s+([A-Za-z0-9]{32,44})",
        // Pattern 4: Any 32-44 character alphanumeric string (fallback)
        r"([A-Za-z0-9]{32,44})",
    ];
    
    for (i, pattern) in token_patterns.iter().enumerate() {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.captures_iter(content) {
                if let Some(addr_match) = cap.get(1) {
                    let addr_str = addr_match.as_str();
                    info!("🔍 Pattern {} matched address: {}", i + 1, addr_str);
                    
                    if let Ok(pubkey) = Pubkey::from_str(addr_str) {
                        info!("✅ Valid pubkey format: {}", pubkey);
                        if is_likely_token_address(&pubkey).await {
                            info!("✅ Token address validated: {}", pubkey);
                            return Some(pubkey);
                        } else {
                            info!("❌ Token address validation failed: {}", pubkey);
                        }
                    } else {
                        info!("❌ Invalid pubkey format: {}", addr_str);
                    }
                }
            }
        }
    }
    
    info!("❌ Signal detected but no valid token address found in: '{}'", content);
    None
}

async fn is_likely_token_address(pubkey: &Pubkey) -> bool {
    // Basic validation - check if it's a valid pubkey format
    if pubkey.to_string().len() != 44 {
        return false;
    }
    
    // TODO: Add more sophisticated validation:
    // - Check if account exists on-chain using RPC
    // - Verify it's actually a token mint account
    // - Check for minimum liquidity using Jupiter
    
    // For now, accept all valid pubkey formats
    // You can enable the advanced validation below when ready
    
    // Uncomment this to add on-chain validation:
    // match validate_token_on_chain(pubkey).await {
    //     Ok(true) => true,
    //     Ok(false) => {
    //         tracing::warn!("Token {} is not a valid mint account", pubkey);
    //         false
    //     }
    //     Err(e) => {
    //         tracing::warn!("Could not validate token {}: {}", pubkey, e);
    //         true // Accept if we can't check (could be RPC issues)
    //     }
    // }
    
    true
}

// Uncomment and configure this function to add on-chain validation
/*
async fn validate_token_on_chain(pubkey: &Pubkey) -> Result<bool, Box<dyn std::error::Error>> {
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_account_decoder::UiAccountEncoding;
    use solana_client::rpc_config::RpcAccountInfoConfig;
    
    // You'll need to add your RPC endpoint here
    let rpc = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    
    // Check if account exists and is a token mint
    match rpc.get_account_with_config(
        pubkey,
        RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..Default::default()
        },
    ).await {
        Ok(response) => {
            if let Some(account) = response.value {
                // Check if it's owned by the token program
                let token_program = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
                return Ok(account.owner == token_program && account.data.len() >= 82); // Mint account size
            }
        }
        Err(_) => return Ok(false),
    }
    
    Ok(false)
}
*/

