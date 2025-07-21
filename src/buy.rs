use anyhow::Result;
use solana_sdk::{pubkey::Pubkey, signer::keypair::Keypair};
use solana_client::nonblocking::rpc_client::RpcClient;

pub async fn execute(mint: Pubkey, cfg: crate::Config, payer: Pubkey) -> Result<()> {
    let rpc = RpcClient::new(cfg.rpc_http.clone());
    tracing::info!("Buying {} with {} SOL", mint, cfg.amount_sol);

    // --- placeholder: craft real pumpfun ix here ---
    let ix = solana_sdk::system_instruction::transfer(
        &payer,
        &mint, // dummy target
        (cfg.amount_sol * 1e9) as u64,
    );
    let bh = rpc.get_latest_blockhash().await?;
    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer),
        &[&Keypair::new()], // signer stub
        bh,
    );
    rpc.send_and_confirm_transaction(&tx).await?;

    crate::notifier::log(format!("🟢 BOUGHT {}", mint)).await;
    tokio::spawn(crate::strategy::manage(mint, cfg, payer));
    Ok(())
}