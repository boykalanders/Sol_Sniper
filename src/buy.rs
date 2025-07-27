use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, transaction::VersionedTransaction};
use std::str::FromStr;
use std::sync::Arc;
use solana_sdk::signer::keypair::Keypair;

pub async fn execute(mint: Pubkey, cfg: crate::Config, payer: Arc<Keypair>) -> Result<()> {
    let rpc = RpcClient::new(cfg.rpc_http.clone());
    tracing::info!("Buying {} with {} SOL", mint, cfg.amount_sol);
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    let amount = (cfg.amount_sol * 1e9_f64) as u64;
    let mut tx = crate::swap::get_swap_transaction(&cfg, &payer.pubkey(), sol_mint, mint, amount).await?;
    let bh = rpc.get_latest_blockhash().await?;
    tx.message.recent_blockhash = bh;
    tx.sign(&[payer.as_ref()], bh);
    rpc.send_and_confirm_transaction(&tx).await?;
    crate::notifier::log(format!("🟢 BOUGHT {}", mint)).await;
    tokio::spawn(crate::strategy::manage(mint, cfg, payer.clone()));
    Ok(())
}