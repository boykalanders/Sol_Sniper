pub async fn execute(mint: Pubkey, sell_pct: u64, cfg: Config) -> Result<()> {
    // same pattern as buy.rs but selling 100 %
    let ix = pumpfun::sell_ix(&mint, sell_pct)?;
    0slot::send(tx, cfg.grpc_x_token).await?;
    notifier::log(format!("🔴 SOLD {mint}")).await;
    Ok(())
}