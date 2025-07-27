use anyhow::Result;

pub async fn log(msg: String) {
    let cfg: crate::Config = toml::from_str(
        &std::fs::read_to_string("config.toml").unwrap()
    ).unwrap();
    let (tg_res, disc_res) = tokio::join!(
        tg(&cfg.tg_token, &cfg.tg_chat, &msg),
        discord(&cfg.discord_webhook, &msg),
    );
    if let Err(e) = tg_res {
        tracing::error!("Failed to send TG notification: {}", e);
    }
    if let Err(e) = disc_res {
        tracing::error!("Failed to send Discord notification: {}", e);
    }
}

async fn tg(token: &str, chat: &str, msg: &str) -> Result<()> {
    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    tracing::debug!("Sending to Telegram URL: {}", url);
    tracing::debug!("Payload: chat_id={}, text={}", chat, msg);
    let response = reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": chat,
            "text": msg
        }))
        .send()
        .await?;
    tracing::debug!("Telegram response status: {}", response.status());
    if !response.status().is_success() {
        let error_text = response.text().await?;
        tracing::error!("Telegram API error: {}", error_text);
        return Err(anyhow::anyhow!("Telegram API failed: {}", error_text));
    }
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