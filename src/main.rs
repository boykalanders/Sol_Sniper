use anyhow::{anyhow, Result};
use serde::Deserialize;
use solana_sdk::{signature::read_keypair_file, signer::Signer};
use std::sync::Arc;
use tracing::info;

mod discord_listener;
mod buy;
mod sell;
mod strategy;
mod notifier;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub rpc_http: String,
    pub grpc_addr: String,
    pub grpc_x_token: String,
    pub tg_token: String,
    pub tg_chat: String,
    pub discord_webhook: String,
    pub discord_token: String,
    pub discord_channel_id: Vec<String>,
    pub amount_sol: f64,
    pub slippage_bps: u16,
    pub priority_fee_microlamports: u64,
    pub take_profit_pct: u32,
    pub stop_loss_pct: u32,
    pub trail_up_50_sl: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg: Config = toml::from_str(&std::fs::read_to_string("config.toml")?)?;
    crate::notifier::log("Test notification on startup".to_string()).await;
    let payer = Arc::new(read_keypair_file("keys/id.json")
        .map_err(|e| anyhow!("bad keypair file: {}", e))?);
    
    // Start both listeners concurrently
    let discord_task = tokio::spawn(discord_listener::run(cfg.clone(), payer.pubkey()));
    
    info!("Started Discord signal monitor");
    
    // Wait for either to finish or Ctrl+C
    tokio::select! {
        _ = discord_task => info!("Discord listener ended"),
        _ = tokio::signal::ctrl_c() => info!("Received Ctrl+C, shutting down"),
    }
    Ok(())
}