use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use tonic::transport::Channel;
use yellowstone_grpc_client::{GeyserGrpcClient, InterceptedService};
use yellowstone_grpc_proto::prelude::{
    geyser_client::GeyserClient, health_client::HealthClient,
    SubscribeRequest, SubscribeRequestFilterTransactions,
};

pub async fn run(cfg: crate::Config, payer: Pubkey) -> Result<()> {
    // Build the channel first
    let channel = Channel::from_shared(cfg.grpc_addr.clone())?
        .connect()
        .await?;

    // Wrap with interceptor (token)
    let interceptor = move |mut req: tonic::Request<()>| {
        req.metadata_mut()
            .insert("x-token", cfg.grpc_x_token.parse().unwrap());
        Ok(req)
    };

    let health = HealthClient::with_interceptor(channel.clone(), interceptor.clone());
    let geyser = GeyserClient::with_interceptor(channel, interceptor);

    let client = GeyserGrpcClient::new(health, geyser);

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
    let logs = tx.transaction.as_ref()?.meta.as_ref()?.log_messages.join(" ");
    logs.find("Mint: ").and_then(|i| logs[i + 6..i + 50].parse().ok())
}