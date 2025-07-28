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

pub async fn run(config: crate::Config, payer: Arc<Keypair>, connected: Arc<AtomicBool>) -> Result<()> {
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

async fn connect_and_listen(config: &crate::Config, payer: Arc<Keypair>, connected: &Arc<AtomicBool>) -> Result<()> {
    let (ws_stream, _) = connect_async("wss://gateway.discord.gg/?v=10&encoding=json")
        .await
        .context("Failed to connect to Discord Gateway")?;
    let (mut write, mut read) = ws_stream.split();
    let token = config.discord_token.clone();
    let channel_ids: Vec<String> = config.discord_channel_id.clone();
    let identify = json!({
        "op": 2,
        "d": {
            "token": token,
            "intents": 512,
            "properties": {
                "$os": "linux",
                "$browser": "custom",
                "$device": "custom"
            }
        }
    });
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
            let event: Value = serde_json::from_str(&text)?;
            match event["op"].as_i64() {
                Some(10) => {
                    let heartbeat_interval = event["d"]["heartbeat_interval"]
                        .as_u64()
                        .unwrap_or(45000) as u64;
                    heartbeat_tx.send(heartbeat_interval).await?;
                    info!("Discord Gateway connected");
                    connected.store(true, Ordering::Relaxed);
                    crate::notifier::log("✅ Discord Gateway connected".to_string()).await;
                }
                Some(0) => {
                    let event_type = event["t"].as_str().unwrap_or("");
                    if event_type == "READY" {
                        info!("Discord Gateway ready");
                    } else if event_type == "MESSAGE_CREATE" {
                        let message = &event["d"];
                        let channel_id = message["channel_id"].as_str().unwrap_or("");
                        if !channel_ids.contains(&channel_id.to_string()) {
                            continue;
                        }
                        if message["author"]["bot"].as_bool().unwrap_or(false) {
                            continue;
                        }
                        let content = message["content"].as_str().unwrap_or("");
                        if let Some(token_address) = parse_trading_signal(content).await {
                            info!("Token address detected: {}", token_address);
                            let config_clone = config.clone();
                            let payer_clone = payer.clone();
                            tokio::spawn(crate::buy::execute(
                                token_address,
                                config_clone,
                                payer_clone,
                            ));
                            let notification = format!(
                                "🚀 Token detected!\nToken: {}\nChannel: {}",
                                token_address, channel_id
                            );
                            crate::notifier::log(notification).await;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    info!("Discord WebSocket loop ended");
    Err(anyhow!("WebSocket disconnected"))
}

async fn parse_trading_signal(content: &str) -> Option<Pubkey> {
    let content = content.to_lowercase();
    let signal_patterns = [r"(?i)\b(CA)\b"];
    let has_signal = signal_patterns.iter().any(|pattern| {
        Regex::new(pattern).unwrap().is_match(&content)
    });
    if !has_signal {
        return None;
    }
    let token_patterns = [
        r"([A-Za-z0-9]{32,44})\b",
        r"(?i)(?:address|contract|token|ca)[:=\s]+([A-Za-z0-9]{32,44})",
        r"(?i)(?:token|contract|address)\s*[:=]?\s*([A-Za-z0-9]{32,44})",
    ];
    for pattern in &token_patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.captures_iter(&content) {
                if let Some(addr_match) = cap.get(1) {
                    let addr_str = addr_match.as_str();
                    if let Ok(pubkey) = Pubkey::from_str(addr_str) {
                        if is_likely_token_address(&pubkey).await {
                            return Some(pubkey);
                        }
                    }
                }
            }
        }
    }
    info!("Signal detected but no valid token address found in: {}", content);
    None
}

async fn is_likely_token_address(_pubkey: &Pubkey) -> bool {
    true
}

#[derive(Debug, Clone)]
pub struct TradingSignal {
    pub token_address: Pubkey,
}