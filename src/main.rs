use anyhow::Result;
use solana_sdk::signature::{read_keypair_file, Signer};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg: crate::Config = toml::from_str(&std::fs::read_to_string("config.toml")?)?;
    let payer = Arc::new(read_keypair_file("keys/id.json")?);
    tokio::spawn(grpc_listener::run(cfg, payer.pubkey()));
    tokio::signal::ctrl_c().await?;
    Ok(())
}