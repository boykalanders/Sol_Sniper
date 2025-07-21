use yellowstone_grpc_client::{GeyserGrpcClient, SubscribeRequestFilterTransactions};
use solana_sdk::{pubkey::Pubkey, transaction::SanitizedTransaction};
use anyhow::*;

pub async fn run(cfg: Config, payer: Pubkey) -> Result<()> {
    let mut client = GeyserGrpcClient::connect(cfg.grpc_addr, cfg.grpc_x_token, None).await?;

    let mut stream = client.subscribe_once(
        SubscribeRequestFilterTransactions {
            account_include: vec![
                "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".into(), // pumpfun
                "BonkFun111111111111111111111111111111111111".into(),   // bonkfun
            ],
            ..Default::default()
        },
    ).await?;

    while let Some(update) = stream.next().await {
        for tx in update.transactions {
            if let Some(ix) = extract_create_ix(&tx) {
                let mint = ix.mint;
                // fire-and-forget buy
                tokio::spawn(buy::execute(mint, cfg.clone(), payer));
            }
        }
    }
    Ok(())
}