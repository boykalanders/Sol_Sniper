use anyhow::{anyhow, Result};
use serde::Deserialize;
use solana_sdk::{signature::read_keypair_file, signer::Signer};
use std::sync::Arc;
use tracing::info;

mod grpc_listener;
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
    pub discord_bot_token: String,
    pub discord_channel_id: String,
    pub amount_sol: f64,
    pub slippage_bps: u16,
    pub priority_fee_microlamports: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg: Config = toml::from_str(&std::fs::read_to_string("config.toml")?)?;
    let payer = Arc::new(read_keypair_file("keys/id.json")
        .map_err(|e| anyhow!("bad keypair file: {}", e))?);
    
    // Start both listeners concurrently
    let grpc_task = tokio::spawn(grpc_listener::run(cfg.clone(), payer.pubkey()));
    let discord_task = tokio::spawn(discord_listener::run(cfg.clone(), payer.pubkey()));
    
    info!("Started GRPC listener and Discord signal monitor");
    
    // Wait for either to finish or Ctrl+C
    tokio::select! {
        _ = grpc_task => info!("GRPC listener ended"),
        _ = discord_task => info!("Discord listener ended"),
        _ = tokio::signal::ctrl_c() => info!("Received Ctrl+C, shutting down"),
    }
    Ok(())
}