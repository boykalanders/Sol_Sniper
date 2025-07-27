use {
    anyhow::{Context, Result},
    futures_util::stream::TryStreamExt,
    solana_sdk::pubkey::Pubkey,
    std::{collections::HashMap, time::Duration},
    tokio::time::timeout,
    tracing::{error, info},
    yellowstone_grpc_client::GeyserGrpcClient,
    yellowstone_grpc_proto::prelude::{
        SubscribeRequest, SubscribeRequestFilterTransactions,
    },
};

pub async fn run(cfg: crate::Config, _payer: Pubkey) -> Result<()> {
    // 1. Build a plain HTTP/2 channel (no TLS, no x-token)
    let mut client = GeyserGrpcClient::build_from_shared(cfg.grpc_addr.clone())?
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(10))
        .max_decoding_message_size(256 * 1024 * 1024)
        .connect()
        .await
        .context("gRPC connect")?;

    info!("Connected to Yellowstone gRPC (whitelisted)");

    // 2. Subscription request
    let req = SubscribeRequest {
        transactions: HashMap::from([(
            "pump_bonk".into(),
            SubscribeRequestFilterTransactions {
                account_include: vec![
                    "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".into(),
                    "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj".into(),
                ],
                vote: Some(false),
                failed: Some(false),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };

    // 3. Start the stream
    let mut stream = client.subscribe_once(req).await.context("subscribe")?;

    // 4. Consume updates
    // Removed mint detection and buy trigger as per user request
    // The listener can be extended for other subscriptions if needed
    loop {
        match timeout(Duration::from_secs(30), stream.try_next()).await {
            Ok(Ok(Some(update))) => {
                // Custom logic can be added here for other gRPC interactions
                // info!("Received gRPC update: {:?}", update);
            }
            Ok(Ok(None)) => {
                error!("Server closed stream");
                break;
            }
            Ok(Err(e)) => {
                error!("Stream error: {}", e);
                break;
            }
            Err(_) => {
                error!("Heartbeat missed");
                break;
            }
        }
    }

    Ok(())
}