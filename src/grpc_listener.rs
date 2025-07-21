use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{
    SubscribeRequest, SubscribeRequestFilterTransactions,
};

pub async fn run(cfg: crate::Config, payer: Pubkey) -> Result<()> {
    let mut client = GeyserGrpcClient::connect(
        cfg.grpc_addr.clone(),
        cfg.grpc_x_token.clone(),
        None,
    )
    .await?;

    let req = SubscribeRequest {
        transactions: {
            let mut m = std::collections::HashMap::new();
            m.insert(
                "pump_bonk".into(),
                SubscribeRequestFilterTransactions {
                    account_include: vec![
                        "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".into(),
                        "BonkFun111111111111111111111111111111111111".into(),
                    ],
                    ..Default::default()
                },
            );
            m
        },
        ..Default::default()
    };

    let mut stream = client.subscribe(req).await?;
    while let Some(update) = stream.next().await {
        for tx in update.transactions {
            if let Some(mint) = extract_mint(&tx) {
                tokio::spawn(crate::buy::execute(mint, cfg.clone(), payer));
            }
        }
    }
    Ok(())
}

fn extract_mint(tx: &yellowstone_grpc_proto::prelude::SubscribeUpdateTransaction) -> Option<Pubkey> {
    // quick-and-dirty log scan
    let logs = tx.transaction.as_ref()?.meta.as_ref()?.log_messages.join(" ");
    logs.find("Mint: ").and_then(|i| {
        let start = i + 6;
        let end = start + 44;
        logs.get(start..end)?.parse().ok()
    })
}