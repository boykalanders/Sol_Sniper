use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use solana_client::nonblocking::rpc_client::RpcClient;

pub async fn execute(mint: Pubkey, cfg: crate::Config, payer: Pubkey) -> Result<()> {
    let rpc = RpcClient::new(cfg.rpc_http.clone());
    tracing::info!("Selling {}", mint);

    // --- placeholder: craft real pumpfun sell ix ---
    let ix = solana_sdk::system_instruction::transfer(
        &payer,
        &mint,
        1_000_000,
    );
    let bh = rpc.get_latest_blockhash().await?;
    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer),
        &[&Keypair::new()],
        bh,
    );
    rpc.send_and_confirm_transaction(&tx).await?;

    crate::notifier::log(format!("🔴 SOLD {}", mint)).await;
    Ok(())
}