use anyhow::{Context, Result};
use regex::Regex;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tracing::{error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::{interval, Interval};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use url::Url;
use std::time::Duration;
use serde_json::Value::Null;

pub async fn run(config: crate::Config, payer: Pubkey, connected: Arc<AtomicBool>) -> Result<()> {
    let gateway_url = Url::parse("wss://gateway.discord.gg/?v=9&encoding=json")?;
    let (ws_stream, _) = connect_async(gateway_url).await.context("Failed to connect to Discord Gateway")?;
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
                if let Err(e) = write.send(Message::Text(json!({"op": 1, "d": null}).to_string())).await {
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
                    let heartbeat_interval = event["d"]["heartbeat_interval"].as_u64().unwrap_or(45000) as u64;
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
                        if let Some(signal) = parse_trading_signal(content).await {
                            info!("Trading signal detected: {:?}", signal);
                            let config_clone = config.clone();
                            let payer_clone = payer;
                            tokio::spawn(crate::buy::execute(signal.token_address, config_clone, payer_clone));
                            let notification = format!(
                                "🚀 Signal detected!\nToken: {}\nSignal: {}\nChannel: {}",
                                signal.token_address,
                                signal.signal_type,
                                channel_id
                            );
                            crate::notifier::log(notification).await;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

async fn parse_trading_signal(content: &str) -> Option<TradingSignal> {
    let re_token = Regex::new(r"0x[a-fA-F0-9]{40}").unwrap();
    let re_signal = Regex::new(r"BUY|SELL").unwrap();

    if let Some(token_match) = re_token.find(content) {
        let token_address = Pubkey::from_str(&token_match.as_str()).unwrap();

        if let Some(signal_match) = re_signal.find(content) {
            let signal_type = signal_match.as_str().to_string();
            return Some(TradingSignal { token_address, signal_type });
        }
    }
    None
}

async fn is_likely_token_address(_pubkey: &Pubkey) -> bool {
    true
}

fn detect_signal_type(content: &str) -> String {
    let re_signal = Regex::new(r"BUY|SELL").unwrap();
    if let Some(signal_match) = re_signal.find(content) {
        return signal_match.as_str().to_string();
    }
    "UNKNOWN".to_string()
}

#[derive(Debug, Clone)]
pub struct TradingSignal {
    pub token_address: Pubkey,
    pub signal_type: String,
} 