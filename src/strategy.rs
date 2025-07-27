use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::time::Duration;
use tokio::time::sleep;
use reqwest;
use serde_json;

pub async fn manage(mint: Pubkey, cfg: crate::Config, payer: Pubkey) -> Result<()> {
    let entry_price = get_token_price_in_sol(&mint).await?;
    let mut max_price: f64 = entry_price;
    let take_profit_multiplier = 1.0 + (cfg.take_profit_pct as f64 / 100.0);
    let stop_loss_multiplier = 1.0 - (cfg.stop_loss_pct as f64 / 100.0);
    let mut trailing_sl = if cfg.trail_up_50_sl { Some(entry_price) } else { None };

    loop {
        sleep(Duration::from_millis(1_000)).await;
        let price = get_token_price_in_sol(&mint).await.unwrap_or(0.0);
        if price == 0.0 {
            tracing::warn!("Failed to fetch price for {}", mint);
            continue;
        }
        max_price = max_price.max(price);

        // Update trailing stop if enabled and we've hit 50% up
        if cfg.trail_up_50_sl && max_price >= entry_price * 1.5 {
            trailing_sl = Some(entry_price); // Set SL to breakeven
        }

        // Check take profit
        if price >= entry_price * take_profit_multiplier {
            crate::sell::execute(mint, cfg.clone(), payer).await?;
            break;
        }
        // Check trailing stop
        if let Some(sl) = trailing_sl {
            if price <= sl {
                crate::sell::execute(mint, cfg.clone(), payer).await?;
                break;
            }
        }
        // Check hard stop loss
        if price <= entry_price * stop_loss_multiplier {
            crate::sell::execute(mint, cfg.clone(), payer).await?;
            break;
        }
    }
    Ok(())
}

async fn get_token_price_in_sol(mint: &Pubkey) -> Result<f64> {
    let client = reqwest::Client::new();
    let url = format!("https://price.jup.ag/v4/price?ids={}&vsToken=So11111111111111111111111111111111111111112", mint);
    let resp = client.get(&url).send().await?;
    let resp = resp.error_for_status()?;
    let json: serde_json::Value = resp.json().await?;
    let price = json["data"][mint.to_string()]["price"].as_f64().unwrap_or(0.0);
    Ok(price)
}