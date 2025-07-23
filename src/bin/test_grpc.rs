use std::{collections::HashMap, time::Duration};
use anyhow::{Context, Result};
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use tracing::{error, info};
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestFilterTransactions};

#[derive(Deserialize)]
struct Config {
    grpc_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg: Config = toml::from_str(&std::fs::read_to_string("config.toml")?)?;
    // Build without x-token
    let mut client = GeyserGrpcClient::build_from_shared(cfg.grpc_addr.clone())?
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(10))
        .connect()
        .await
        .context("gRPC connect")?;

    info!("Connected to Yellowstone gRPC without x-token");

    // Subscription request to test auth
    let req = SubscribeRequest {
        transactions: HashMap::from([(
            "test".into(),
            SubscribeRequestFilterTransactions {
                account_include: vec![
                    "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".into(),
                ],
                ..Default::default()
            },
        )]),
        ..Default::default()
    };

    let _stream = client.subscribe_once(req).await.context("subscribe")?;

    info!("Successfully subscribed - your connection is whitelisted");

    Ok(())
} 