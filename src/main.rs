use anyhow::{anyhow, Result};
use serde::Deserialize;
use solana_sdk::{signature::read_keypair_file, pubkey::Pubkey, signer::keypair::Keypair};
use solana_sdk::signer::Signer;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;
use tracing::info;
use std::sync::atomic::{AtomicBool, Ordering};

mod discord_listener;
mod buy;
mod sell;
mod strategy;
mod notifier;
mod swap;
mod grpc_listener;
mod profit_db;
mod telegram_bot;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub rpc_http: String,
    pub grpc_addr: String,
    pub grpc_x_token: String,
    pub tg_token: String,
    pub tg_chat: String,
    pub tg_authorized_users: Vec<String>,
    pub discord_webhook: String,
    pub discord_token: String,
    pub discord_channel_id: Vec<String>,
    pub amount_sol: f64,
    pub slippage_bps: u16,
    pub priority_fee_microlamports: u64,
    pub take_profit_pct: u32,
    pub stop_loss_pct: u32,
    pub trail_up_50_sl: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg: Config = toml::from_str(&std::fs::read_to_string("config.toml")?)?;
    
    // Initialize profit database
    let profit_db = match profit_db::ProfitDatabase::new("profit_tracking.db") {
        Ok(db) => {
            info!("✅ Profit tracking database initialized");
            db
        }
        Err(e) => {
            tracing::error!("Failed to initialize profit database: {}", e);
            return Err(anyhow!("Failed to initialize profit database: {}", e));
        }
    };
    
    // Initialize Telegram bot controller
    let telegram_controller = telegram_bot::TelegramController::new(
        cfg.tg_token.clone(),
        profit_db.clone(),
        cfg.tg_authorized_users.clone(),
    );
    
    crate::notifier::log("Test notification on startup".to_string()).await;
    let payer = Arc::new(read_keypair_file("keys/id.json")
        .map_err(|e| anyhow!("bad keypair file: {}", e))?);
    
    // Display wallet info and balance
    info!("💰 Wallet Address: {}", payer.pubkey());
    info!("💵 Trading with {} SOL per signal", cfg.amount_sol);
    
    // Check current SOL balance
    match get_sol_balance(&cfg.rpc_http, &payer.pubkey()).await {
        Ok(balance) => {
            info!("💰 Current SOL Balance: {:.4} SOL", balance);
            let trades_possible = (balance / cfg.amount_sol).floor() as u32;
            info!("📊 Trades possible with current balance: {}", trades_possible);
            
            if balance < cfg.amount_sol {
                let msg = format!("⚠️ WARNING: Balance ({:.4} SOL) is less than trade amount ({} SOL)", balance, cfg.amount_sol);
                tracing::warn!("{}", msg);
                crate::notifier::log(msg).await;
            } else if balance < cfg.amount_sol * 3.0 {
                let msg = format!("💛 Low balance warning: Only {:.4} SOL remaining ({}x trades possible)", balance, trades_possible);
                tracing::warn!("{}", msg);
                crate::notifier::log(msg).await;
            }
            
            crate::notifier::log(format!("💰 Bot started | Wallet: {} | Balance: {:.4} SOL", payer.pubkey(), balance)).await;
        }
        Err(e) => {
            tracing::warn!("Failed to get SOL balance: {}", e);
            crate::notifier::log(format!("💰 Bot started with wallet: {} (balance check failed)", payer.pubkey())).await;
        }
    }
    
    let connected = Arc::new(AtomicBool::new(false));
    let discord_task = tokio::spawn(discord_listener::run(cfg.clone(), payer.clone(), connected.clone()));
    
    info!("Started Discord signal monitor");
    
    // Start Telegram bot
    let telegram_controller_clone = telegram_controller.clone();
    let telegram_task = tokio::spawn(async move {
        if let Err(e) = telegram_controller_clone.start().await {
            tracing::error!("Telegram bot failed: {}", e);
        }
    });
    
    info!("Started Telegram bot controller");
    
    let connected_clone = connected.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        if !connected_clone.load(Ordering::Relaxed) {
            crate::notifier::log("❌ Discord connection timeout - check token and network".to_string()).await;
            tracing::error!("Discord connection timeout");
        }
    });

    let grpc_task = tokio::spawn(grpc_listener::run(cfg.clone(), payer.pubkey()));

    // Start periodic balance monitor (every 5 minutes)
    let balance_monitor = tokio::spawn(periodic_balance_monitor(cfg.clone(), payer.clone()));

    // Wait for either to finish or Ctrl+C
    tokio::select! {
        _ = discord_task => info!("Discord listener ended"),
        _ = grpc_task => info!("gRPC listener ended"),
        _ = telegram_task => info!("Telegram bot ended"),
        _ = balance_monitor => info!("Balance monitor ended"),
        _ = tokio::signal::ctrl_c() => info!("Received Ctrl+C, shutting down"),
    }
    Ok(())
}

/// Get SOL balance for a given wallet address
pub async fn get_sol_balance(rpc_url: &str, wallet: &Pubkey) -> Result<f64> {
    let rpc = RpcClient::new(rpc_url.to_string());
    let balance_lamports = rpc.get_balance(wallet).await?;
    let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
    Ok(balance_sol)
}

/// Periodic balance monitor - logs balance every 5 minutes
async fn periodic_balance_monitor(cfg: Config, payer: Arc<Keypair>) -> Result<()> {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
    let mut last_balance = 0.0;
    
    loop {
        interval.tick().await;
        
        match get_sol_balance(&cfg.rpc_http, &payer.pubkey()).await {
            Ok(current_balance) => {
                let balance_change = current_balance - last_balance;
                
                if last_balance > 0.0 {
                    if balance_change.abs() > 0.001 { // Only log if change is significant
                        let change_str = if balance_change > 0.0 {
                            format!("+{:.4}", balance_change)
                        } else {
                            format!("{:.4}", balance_change)
                        };
                        
                        tracing::info!("📊 Balance Update: {:.4} SOL ({})", current_balance, change_str);
                        
                        // Warn if balance is getting low
                        let trades_possible = (current_balance / cfg.amount_sol).floor() as u32;
                        if current_balance < cfg.amount_sol * 2.0 && current_balance > cfg.amount_sol {
                            crate::notifier::log(format!("💛 Balance getting low: {:.4} SOL ({} trades left)", current_balance, trades_possible)).await;
                        }
                    }
                } else {
                    tracing::debug!("📊 Periodic balance check: {:.4} SOL", current_balance);
                }
                
                last_balance = current_balance;
            }
            Err(e) => {
                tracing::debug!("Failed periodic balance check: {}", e);
            }
        }
    }
}