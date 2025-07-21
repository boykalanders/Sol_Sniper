use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::geyser::{
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
                        "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".into(), // pumpfun
                        "BonkFun111111111111111111111111111111111111".into(),   // bonkfun
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

fn extract_mint(tx: &yellowstone_grpc_proto::geyser::SubscribeUpdateTransaction) -> Option<Pubkey> {
    // naive log scanner – replace with real parser
    let log = tx.transaction.meta.as_ref()?.log_messages.join(" ");
    let re = regex::Regex::new(r"Mint: (\w{44})").ok()?;
    let cap = re.captures(&log)?;
    cap.get(1)?.as_str().parse().ok()
}