use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::time::Duration;
use tokio::time::sleep;

pub async fn manage(mint: Pubkey, cfg: crate::Config, payer: Pubkey) -> Result<()> {
    let rpc = RpcClient::new(cfg.rpc_http.clone());
    let entry_price = 1.0; // fetch real price
    let mut max_price = entry_price;

    loop {
        sleep(Duration::from_millis(1_000)).await;
        let price = 1.0; // fetch real price
        max_price = max_price.max(price);

        if price >= entry_price * 5.0 {
            crate::sell::execute(mint, cfg.clone(), payer).await?;
            break;
        }
        if max_price >= entry_price * 1.5 && price <= entry_price {
            crate::sell::execute(mint, cfg.clone(), payer).await?;
            break;
        }
        if price <= entry_price * 0.35 {
            crate::sell::execute(mint, cfg.clone(), payer).await?;
            break;
        }
    }
    Ok(())
}