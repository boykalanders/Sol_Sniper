pub async fn manage(mint: Pubkey, cfg: Config) -> Result<()> {
    let rpc = RpcClient::new(cfg.rpc_http.clone());
    let entry_price = pumpfun::price(&mint, &rpc).await?;
    let mut max_price = entry_price;
    let mut stop = entry_price * 0.35;         // 65 % SL

    loop {
        tokio::time::sleep(Duration::from_millis(1000)).await;
        let price = pumpfun::price(&mint, &rpc).await?;

        // rule 1: never hit 50 % up → 65 % SL
        if price > entry_price * 1.5 {
            stop = entry_price;                // rule 2: lock initial cost
        }

        // rule 3: 5 x take profit
        if price >= entry_price * 5.0 {
            sell::execute(mint, 100, cfg.clone()).await?;
            break;
        }

        // trailing SL hit
        if price <= stop {
            sell::execute(mint, 100, cfg.clone()).await?;
            break;
        }

        // update max
        if price > max_price {
            max_price = price;
        }
    }
    Ok(())
}