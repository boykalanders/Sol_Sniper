use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey /*, transaction::VersionedTransaction */};
use std::str::FromStr;
use std::sync::Arc;
use solana_sdk::{signer::keypair::Keypair, signer::Signer};

pub async fn execute(mint: Pubkey, cfg: crate::Config, payer: Arc<Keypair>) -> Result<()> {
    let rpc = RpcClient::new(cfg.rpc_http.clone());
    tracing::info!("🎯 Signal received: attempting to buy {} with {} SOL", mint, cfg.amount_sol);
    
    // Check current balance before trade
    match crate::get_sol_balance(&cfg.rpc_http, &payer.pubkey()).await {
        Ok(balance) => {
            tracing::info!("💰 Current balance before trade: {:.4} SOL", balance);
            if balance < cfg.amount_sol {
                let msg = format!("❌ Insufficient balance: {:.4} SOL < {} SOL needed", balance, cfg.amount_sol);
                tracing::error!("{}", msg);
                crate::notifier::log(msg).await;
                return Ok(());
            }
        }
        Err(e) => {
            tracing::warn!("Could not check balance before trade: {}", e);
        }
    }
    
    // Check token liquidity before attempting to buy
    tracing::info!("Checking liquidity for token {}...", mint);
    match crate::swap::check_token_liquidity(&mint, cfg.amount_sol).await {
        Ok(true) => {
            tracing::info!("✅ Token {} has sufficient liquidity", mint);
        }
        Ok(false) => {
            let msg = format!("❌ Token {} has insufficient liquidity - skipping", mint);
            tracing::warn!("{}", msg);
            crate::notifier::log(msg).await;
            return Ok(());
        }
        Err(e) => {
            tracing::warn!("Could not check liquidity for {}: {}. Proceeding anyway...", mint, e);
        }
    }
    
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?;
    let amount = (cfg.amount_sol * 1e9_f64) as u64;
    
    tracing::info!("Getting swap transaction from Jupiter...");
    let mut tx = match crate::swap::get_swap_transaction(&cfg, &payer.pubkey(), sol_mint, mint, amount).await {
        Ok(tx) => tx,
        Err(e) => {
            let msg = format!("❌ Failed to get swap route for {}: {}", mint, e);
            tracing::error!("{}", msg);
            crate::notifier::log(msg).await;
            return Err(e);
        }
    };
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
    
    tracing::info!("Sending transaction to buy {}...", mint);
    match rpc.send_and_confirm_transaction(&tx).await {
        Ok(signature) => {
            tracing::info!("✅ Successfully bought {}, signature: {}", mint, signature);
            
            // Check balance after successful trade
            match crate::get_sol_balance(&cfg.rpc_http, &payer.pubkey()).await {
                Ok(new_balance) => {
                    tracing::info!("💰 Balance after trade: {:.4} SOL", new_balance);
                    let spent_amount = cfg.amount_sol; // Approximate, actual might vary due to fees
                    crate::notifier::log(format!("🟢 BOUGHT {} | TX: {} | Balance: {:.4} SOL", mint, signature, new_balance)).await;
                }
                Err(e) => {
                    tracing::warn!("Could not check balance after trade: {}", e);
                    crate::notifier::log(format!("🟢 BOUGHT {} - TX: {}", mint, signature)).await;
                }
            }
            
            tokio::spawn(crate::strategy::manage(mint, cfg, payer.clone()));
            Ok(())
        }
        Err(e) => {
            let msg = format!("❌ Transaction failed for {}: {}", mint, e);
            tracing::error!("{}", msg);
            crate::notifier::log(msg).await;
            Err(e.into())
        }
    }
}