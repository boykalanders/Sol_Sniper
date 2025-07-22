use anyhow::Result;
use futures_util::stream::TryStreamExt;
use solana_sdk::pubkey::Pubkey;
use yellowstone_grpc_client::{GeyserGrpcClient, Interceptor};
use yellowstone_grpc_proto::prelude::{
    subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestFilterTransactions,
};
use tracing::info;

pub async fn run(cfg: crate::Config, payer: Pubkey) -> Result<()> {
    let grpc_addr = cfg.grpc_addr.clone();
    info!("Connecting to gRPC at {}", grpc_addr);
    let mut client = GeyserGrpcBuilder::build_from_shared(grpc_addr)?
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(10))
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .max_decoding_message_size(1024 * 1024 * 1024)
        .connect()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to gRPC: {}", e))?;
    info!("Connected to gRPC");

    let req = SubscribeRequest {
        transactions: {
            let mut m = std::collections::HashMap::new();
            m.insert(
                "pump_bonk".into(),
                SubscribeRequestFilterTransactions {
                    account_include: vec![
                        "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".into(),
                        "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj".into(),
                    ],
                    ..Default::default()
                },
            );
            m
        },
        ..Default::default()
    };

    let mut stream = client.subscribe_once(req).await?;
    while let Ok(Some(update)) = stream.try_next().await {
        if let Some(UpdateOneof::Transaction(tx)) = update.update_oneof {
            if let Some(mint) = extract_mint(&tx) {
                info!("Found mint: {}", mint);
                tokio::spawn(crate::buy::execute(mint, cfg.clone(), payer));
            }
        }
    }
    Ok(())
}

fn extract_mint(tx: &yellowstone_grpc_proto::prelude::SubscribeUpdateTransaction) -> Option<Pubkey> {
    tx.transaction.as_ref()?.meta.as_ref()?.post_token_balances.first()?
        .mint
        .parse()
        .ok()
}