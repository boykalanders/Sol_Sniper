use anyhow::{anyhow, Result};
use reqwest;
use serde_json::{json, Value};
use solana_sdk::{pubkey::Pubkey, transaction::VersionedTransaction};
use base64;
use bincode;

pub async fn get_swap_transaction(
    cfg: &crate::Config,
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
    let quote_response = client.get(quote_url).send().await?.error_for_status()?.json::<Value>().await?;
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
    let tx_bytes = base64::decode(swap_tx_b64)?;
    let tx: VersionedTransaction = bincode::deserialize(&tx_bytes)?;
    Ok(tx)
} 