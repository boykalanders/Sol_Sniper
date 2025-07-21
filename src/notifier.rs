pub async fn log(msg: String) {
    let cfg = load_config();
    let f1 = tg::send(cfg.tg_token, cfg.tg_chat, &msg);
    let f2 = discord::webhook(cfg.discord_webhook, &msg);
    let _ = tokio::join!(f1, f2);
}