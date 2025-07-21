pub async fn execute(mint: Pubkey, cfg: Config, payer: Keypair) -> Result<()> {
    let rpc = RpcClient::new(cfg.rpc_http.clone());
    let amount = sol_to_lamports(cfg.amount_sol);

    // craft pumpfun buy instruction
    let ix = pumpfun::buy_ix(&mint, amount, cfg.slippage_bps)?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        rpc.get_latest_blockhash().await?,
    );

    // send via 0slot gRPC relayer
    0slot::send(tx, cfg.grpc_x_token).await?;
    notifier::log(format!("🟢 SNIPED {mint}")).await;

    // start trailing-stop
    tokio::spawn(strategy::manage(mint, cfg));
    Ok(())
}