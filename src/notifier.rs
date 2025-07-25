use anyhow::Result;

pub async fn log(msg: String) {
    let cfg: crate::Config = toml::from_str(
        &std::fs::read_to_string("config.toml").unwrap()
    ).unwrap();
    let _ = tokio::join!(
        tg(&cfg.tg_token, &cfg.tg_chat, &msg),
        discord(&cfg.discord_webhook, &msg),
    );
}

async fn tg(token: &str, chat: &str, msg: &str) -> Result<()> {
    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": chat,
            "text": msg
        }))
        .send()
        .await?;
    Ok(())
}

async fn discord(webhook: &str, msg: &str) -> Result<()> {
    reqwest::Client::new()
        .post(webhook)
        .json(&serde_json::json!({ "content": msg }))
        .send()
        .await?;
    Ok(())
}