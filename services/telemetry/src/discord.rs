use std::collections::HashMap;

async fn send_message_internal(msg: String) -> anyhow::Result<()> {
    if let Some(url) = option_env!("DISCORD_TELEMETRY_WEBHOOK") {
        let mut map = HashMap::new();
        map.insert("content", msg.to_string());

        let client = reqwest::Client::new();
        let response = client.post(url).json(&map).send().await?;
        let _ = response.error_for_status()?;
    } else {
        log::info!("Would have sent Discord message: {}", msg);
    }
    Ok(())
}

pub fn send_message(msg: String) {
    tokio::spawn(async move {
        if let Err(e) = send_message_internal(msg).await {
            log::warn!("Failed to send Discord message: {}", e);
        }
    });
}
