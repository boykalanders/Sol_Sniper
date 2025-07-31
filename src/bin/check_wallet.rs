#!/usr/bin/env rust-script

//! Simple utility to check wallet information
//! 
//! Usage: cargo run --bin check_wallet

use anyhow::{anyhow, Result};
use solana_sdk::{signature::read_keypair_file, signer::Signer};
use solana_client::nonblocking::rpc_client::RpcClient;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Checking wallet information...\n");

    // Load the keypair
    let payer = read_keypair_file("keys/id.json")
        .map_err(|e| anyhow!("Failed to read keypair: {}", e))?;

    let wallet_address = payer.pubkey();
    println!("💰 Wallet Address: {}", wallet_address);

    // Connect to RPC and get balance
    let rpc = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    
    match rpc.get_balance(&wallet_address).await {
        Ok(balance_lamports) => {
            let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
            println!("💵 Current Balance: {} SOL ({} lamports)", balance_sol, balance_lamports);
            
            if balance_sol < 0.01 {
                println!("⚠️  Warning: Low balance! Consider depositing more SOL for trading.");
            } else if balance_sol < 0.1 {
                println!("💛 Balance is getting low. Consider topping up.");
            } else {
                println!("✅ Good balance for trading!");
            }
        }
        Err(e) => {
            println!("❌ Failed to get balance: {}", e);
            println!("💡 This might be due to RPC rate limits or network issues.");
        }
    }

    println!("\n📋 Quick Actions:");
    println!("• View on Solscan: https://solscan.io/account/{}", wallet_address);
    println!("• View on Solana Explorer: https://explorer.solana.com/address/{}", wallet_address);
    println!("• Send SOL to this address to fund your trading bot");

    Ok(())
}