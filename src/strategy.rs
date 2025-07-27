use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use solana_sdk::signer::keypair::Keypair;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{SubscribeRequest, SubscribeRequestFilterAccounts};
use futures_util::stream::TryStreamExt;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error};
use bincode;

#[derive(bincode::Decode, Debug)]
struct BondingCurve {
    virtual_sol_reserves: u64,
    virtual_token_reserves: u64,
    // Other fields as needed
}

pub async fn manage(mint: Pubkey, cfg: crate::Config, payer: Arc<Keypair>) -> Result<()> {
    // Derive bonding curve PDA for Pump.fun
    let pump_program = Pubkey::from_str("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P")?;
    let (bonding_curve, _) = Pubkey::find_program_address(&[b"bonding-curve", mint.as_ref()], &pump_program);

    // Get initial price (using existing method or calculate)
    let entry_price = get_initial_price(&bonding_curve, &cfg).await?;
    let mut max_price: f64 = entry_price;
    let trail_multiplier = 1.0 - (cfg.stop_loss_pct as f64 / 100.0);
    let mut sl = entry_price * trail_multiplier;

    // Set up gRPC subscription to bonding curve account
    let mut client = GeyserGrpcClient::build_from_shared(cfg.grpc_addr.clone())?
        .x_token(Some(cfg.grpc_x_token.clone()))?
        .connect().await?;

    let req = SubscribeRequest {
        accounts: HashMap::from([(
            "bonding".to_string(),
            SubscribeRequestFilterAccounts {
                account: vec![bonding_curve.to_string()],
                ..Default::default()
            },
        )]),
        ..Default::default()
    };

    let mut stream = client.subscribe_once(req).await?;

    loop {
        match timeout(Duration::from_secs(30), stream.try_next()).await {
            Ok(Ok(Some(update))) => {
                if let Some(account_update) = update.update_oneof {
                    if let yellowstone_grpc_proto::prelude::subscribe_update::UpdateOneof::Account(acc) = account_update {
                        let data = acc.account.ok_or(anyhow!("No account in update"))?.data.clone();
                        if let Ok((curve, _)) = bincode::decode_from_slice(&data, bincode::config::standard()) {
                            let price = curve.virtual_sol_reserves as f64 / curve.virtual_token_reserves as f64;
                            max_price = max_price.max(price);
                            sl = sl.max(max_price * trail_multiplier);
                            if price <= sl {
                                crate::sell::execute(mint, cfg.clone(), payer.clone()).await?;
                                break;
                            }
                        }
                    }
                }
            }
            Ok(Ok(None)) => { error!("Stream closed"); break; }
            Ok(Err(e)) => { error!("Error: {}", e); break; }
            Err(_) => { error!("Timeout"); break; }
        }
    }
    Ok(())
}

async fn get_initial_price(bonding_curve: &Pubkey, cfg: &crate::Config) -> Result<f64> {
    // Implement initial price fetch, perhaps using RPC
    let rpc = solana_client::nonblocking::rpc_client::RpcClient::new(cfg.rpc_http.clone());
    let data = rpc.get_account_data(bonding_curve).await?;
    let (curve, _) = bincode::decode_from_slice(&data, bincode::config::standard())?;
    Ok(curve.virtual_sol_reserves as f64 / curve.virtual_token_reserves as f64)
}