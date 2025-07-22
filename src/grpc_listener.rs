use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use yellowstone_grpc_client::{GeyserGrpcBuilder};
use yellowstone_grpc_proto::prelude::{
    SubscribeRequest, SubscribeRequestFilterTransactions,
};
use futures::stream::{StreamExt, TryStreamExt};

pub async fn run(cfg: crate::Config, payer: Pubkey) -> Result<()> {
    // 8.0 builder pattern
    let mut client = GeyserGrpcBuilder::from_shared(cfg.grpc_addr)?
        .connect()
        .await?;

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
        for tx in update.transactions {
            if let Some(mint) = extract_mint(&tx) {
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