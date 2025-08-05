use anyhow::Result;
use anyhow::anyhow;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use solana_sdk::signer::keypair::Keypair;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{SubscribeRequest, SubscribeRequestFilterAccounts};
use futures_util::stream::TryStreamExt;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info};
use bincode;
use serde::Deserialize;
use crate::Config;

#[derive(Deserialize, Debug)]
struct BondingCurve {
    virtual_sol_reserves: u64,
    virtual_token_reserves: u64,
    // Other fields as needed
}

pub async fn manage(mint: Pubkey, cfg: Config, payer: Arc<Keypair>) -> Result<()> {
    let bonk_program: Pubkey = "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj".parse()?;
    let (bonding_curve, _) = Pubkey::find_program_address(&[b"bonding-curve", mint.as_ref()], &bonk_program);
    let stop_loss_multiplier = 1.0 - (cfg.stop_loss_pct as f64 / 100.0);
    let take_profit_multiplier = 1.0 + (cfg.take_profit_pct as f64 / 100.0);
    let breakeven_trigger_multiplier = 1.5;
    let mut entry_price: f64 = 0.0;
    let mut sl: f64 = 0.0;
    let mut initialized = false;
    let mut hit_breakeven = false;
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
                        if let Ok(curve) = bincode::deserialize::<BondingCurve>(&data) {
                            let price = curve.virtual_sol_reserves as f64 / curve.virtual_token_reserves as f64;
                            if !initialized {
                                entry_price = price;
                                sl = entry_price * stop_loss_multiplier;
                                initialized = true;
                                continue;
                            }
                            let ratio = price / entry_price;
                            if ratio >= take_profit_multiplier {
                                info!("Take profit triggered at {}x", take_profit_multiplier);
                                crate::sell::execute(mint, cfg.clone(), payer.clone()).await?;
                                break;
                            }
                            if cfg.trail_up_50_sl && !hit_breakeven && ratio >= breakeven_trigger_multiplier {
                                sl = entry_price;
                                hit_breakeven = true;
                                info!("Breakeven SL set at entry price after hitting 1.5x");
                            }
                            // No further trailing after breakeven
                            if price <= sl {
                                info!("Stop loss triggered at price {}", price);
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
