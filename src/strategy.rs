use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use std::time::Duration;
use tokio::time::sleep;
use reqwest;
use serde_json::Value;
use std::sync::Arc;
use solana_sdk::signer::keypair::Keypair;

pub async fn manage(mint: Pubkey, cfg: crate::Config, payer: Arc<Keypair>) -> Result<()> {
    let entry_price = get_token_price_in_sol(&mint).await?;
    let mut max_price: f64 = entry_price;
    let _take_profit_multiplier = 1.0 + (cfg.take_profit_pct as f64 / 100.0);
    let trail_multiplier = 1.0 - (cfg.stop_loss_pct as f64 / 100.0);
    let mut sl = entry_price * trail_multiplier;

    loop {
        sleep(Duration::from_millis(1_000)).await;
        let price = get_token_price_in_sol(&mint).await.unwrap_or(0.0);
        if price == 0.0 {
            tracing::warn!("Failed to fetch price for {}", mint);
            continue;
        }
        max_price = max_price.max(price);
        sl = sl.max(max_price * trail_multiplier);

        // Comment out take profit to only use trailing stop
        // if price >= entry_price * take_profit_multiplier {
        //     crate::sell::execute(mint, cfg.clone(), payer.clone()).await?;
        //     break;
        // }

        // Check trailing stop
        if price <= sl {
            crate::sell::execute(mint, cfg.clone(), payer.clone()).await?;
            break;
        }

        // Removed hard stop loss as it's incorporated in trailing
    }
    Ok(())
}

async fn get_token_price_in_sol(mint: &Pubkey) -> Result<f64> {
    let client = reqwest::Client::new();
    let url = format!("https://price.jup.ag/v4/price?ids={}&vsToken=So11111111111111111111111111111111111111112", mint);
    let resp = client.get(&url).send().await?;
    let resp = resp.error_for_status()?;
    let json: Value = resp.json().await?;
    let price = json["data"][mint.to_string()]["price"].as_f64().unwrap_or(0.0);
    Ok(price)
}