use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, transaction::VersionedTransaction};
use std::str::FromStr;
use std::sync::Arc;
use solana_sdk::signer::keypair::Keypair;
use spl_associated_token_account::get_associated_token_address;

pub async fn execute(mint: Pubkey, cfg: crate::Config, payer: Arc<Keypair>) -> Result<()> {
    let rpc = RpcClient::new(cfg.rpc_http.clone());
    tracing::info!("Selling {}", mint);
    let token_account = get_associated_token_address(&payer.pubkey(), &mint);
    let balance_resp = rpc.get_token_account_balance(&token_account).await?;
    let amount = balance_resp.amount.parse::<u64>()?;
    if amount == 0 {
        tracing::info!("No balance to sell for {}", mint);
        return Ok(());
    }
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    let mut tx = crate::swap::get_swap_transaction(&cfg, &payer.pubkey(), mint, sol_mint, amount).await?;
    let bh = rpc.get_latest_blockhash().await?;
    tx.message.recent_blockhash = bh;
    tx.sign(&[payer.as_ref()], bh);
    rpc.send_and_confirm_transaction(&tx).await?;
    crate::notifier::log(format!("🔴 SOLD {}", mint)).await;
    Ok(())
}