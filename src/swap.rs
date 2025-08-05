use anyhow::{anyhow, Result};
use reqwest;
use serde_json::{json, Value};
use solana_sdk::{pubkey::Pubkey, transaction::VersionedTransaction};
use base64::{self, engine::general_purpose};
use base64::Engine;
use bincode;
use std::str::FromStr;
use crate::Config;

pub async fn get_swap_transaction(
    cfg: &Config,
    payer: &Pubkey,
    input_mint: Pubkey,
    output_mint: Pubkey,
    amount: u64,
) -> Result<VersionedTransaction> {
    let client = reqwest::Client::new();
    let quote_url = format!(
        "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
        input_mint, output_mint, amount, cfg.slippage_bps
    );
    
    tracing::info!("Requesting quote from Jupiter: {}", quote_url);
    let quote_response = client.get(quote_url).send().await?.error_for_status()?.json::<Value>().await?;
    
    // Check if Jupiter found any routes (pools)
    if quote_response.get("routePlan").and_then(|v| v.as_array()).map_or(true, |arr| arr.is_empty()) {
        return Err(anyhow!("No liquidity pools found for token {} -> {}", input_mint, output_mint));
    }
    
    tracing::info!("Jupiter found route with {} steps", 
        quote_response["routePlan"].as_array().map_or(0, |arr| arr.len()));
    
    let expected_out = quote_response["outAmount"].as_str().unwrap_or("0");
    tracing::info!("Expected output amount: {}", expected_out);
    let swap_request = json!({
        "quoteResponse": quote_response,
        "userPublicKey": payer.to_string(),
        "wrapAndUnwrapSol": true,
        "computeUnitPriceMicroLamports": cfg.priority_fee_microlamports
    });
    let swap_response = client.post("https://quote-api.jup.ag/v6/swap")
        .json(&swap_request)
        .send().await?.error_for_status()?.json::<Value>().await?;
    let swap_tx_b64 = swap_response["swapTransaction"].as_str().ok_or(anyhow!("Missing swapTransaction"))?.to_string();
    let tx_bytes = general_purpose::STANDARD.decode(swap_tx_b64)?;
    let tx: VersionedTransaction = bincode::deserialize(&tx_bytes)?;
    Ok(tx)
}

/// Check if a token has sufficient liquidity for trading
pub async fn check_token_liquidity(token_mint: &Pubkey, _min_liquidity_sol: f64) -> Result<bool> {
    let client = reqwest::Client::new();
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    let amount_lamports = (0.001 * 1e9) as u64; // Test with 0.001 SOL
    
    let quote_url = format!(
        "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=300",
        sol_mint, token_mint, amount_lamports
    );
    
    match client.get(quote_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(quote_response) = response.json::<Value>().await {
                    // Check if we have routes and reasonable output
                    if let Some(route_plan) = quote_response.get("routePlan").and_then(|v| v.as_array()) {
                        if !route_plan.is_empty() {
                            if let Some(out_amount) = quote_response.get("outAmount").and_then(|v| v.as_str()) {
                                if let Ok(output_tokens) = out_amount.parse::<u64>() {
                                    // Check if we get reasonable output (indicates good liquidity)
                                    tracing::info!("Token {} liquidity check: {} tokens for 0.001 SOL", 
                                        token_mint, output_tokens);
                                    return Ok(output_tokens > 0);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to check liquidity for {}: {}", token_mint, e);
        }
    }
    
    Ok(false)
} 