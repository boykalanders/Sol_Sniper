use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey /*, transaction::VersionedTransaction */};
use solana_sdk::{hash::Hash, message::{Message, v0}};
use std::str::FromStr;
use std::sync::Arc;
use solana_sdk::{signer::keypair::Keypair, signer::Signer};

pub async fn execute(mint: Pubkey, cfg: crate::Config, payer: Arc<Keypair>) -> Result<()> {
    let rpc = RpcClient::new(cfg.rpc_http.clone());
    tracing::info!("Selling {}", mint);

    let token_program = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?;
    let ata_program = Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")?;
    let token_account = Pubkey::find_program_address(
        &[&payer.pubkey().to_bytes(), token_program.as_ref(), mint.as_ref()],
        &ata_program
    ).0;

    let balance_resp = rpc.get_token_account_balance(&token_account).await?;
    let amount = balance_resp.amount.parse::<u64>()?;
    if amount == 0 {
        tracing::info!("No balance to sell for {}", mint);
        return Ok(());
    }
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    let mut tx = crate::swap::get_swap_transaction(&cfg, &payer.pubkey(), mint, sol_mint, amount).await?;
    let bh = rpc.get_latest_blockhash().await?;
    let mut message = tx.message.clone();
    match &mut message {
        solana_sdk::message::VersionedMessage::Legacy(ref mut msg) => {
            msg.recent_blockhash = bh;
        }
        solana_sdk::message::VersionedMessage::V0(ref mut msg) => {
            msg.recent_blockhash = bh;
        }
    }
    let message_hash = message.hash();
    let sig = payer.sign_message(message_hash.as_ref());
    tx.message = message;
    tx.signatures = vec![sig];
    rpc.send_and_confirm_transaction(&tx).await?;
    crate::notifier::log(format!("🔴 SOLD {}", mint)).await;
    Ok(())
}