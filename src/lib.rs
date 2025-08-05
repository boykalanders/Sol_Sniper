use anyhow::Result;
use serde::Deserialize;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

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

/// Get SOL balance for a given wallet address
pub async fn get_sol_balance(rpc_url: &str, wallet: &Pubkey) -> Result<f64> {
    let rpc = RpcClient::new(rpc_url.to_string());
    let balance_lamports = rpc.get_balance(wallet).await?;
    let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
    Ok(balance_sol)
}

pub mod discord_listener;
pub mod buy;
pub mod sell;
pub mod strategy;
pub mod notifier;
pub mod swap;
pub mod grpc_listener;
pub mod profit_db;
pub mod telegram_bot; 