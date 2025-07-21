use std::sync::Arc;
use solana_sdk::signature::{Keypair, read_keypair_file};
use tracing::info;

mod strategy;
mod grpc_listener;
mod buy;
mod sell;
mod notifier;

#[derive(serde::Deserialize, Clone)]
pub struct Config {
    pub rpc_http: String,
    pub grpc_addr: String,
    pub grpc_x_token: String,
    pub tg_token: String,
    pub tg_chat: String,
    pub discord_webhook: String,
    pub amount_sol: f64,
    pub slippage_bps: u16,
    pub priority_fee_microlamports: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cfg: Config = toml::from_str(&std::fs::read_to_string("config.toml")?)?;
    let payer = Arc::new(read_keypair_file("keys/id.json")?);
    info!("Starting sniper…");
    grpc_listener::run(cfg, payer).await?;
    Ok(())
}