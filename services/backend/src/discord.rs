use std::collections::HashMap;

pub enum Channel {
    Leaderboard,
    Telemetry,
}

fn webhook(channel: Channel) -> Option<&'static str> {
    if let Some(url) = match channel {
        Channel::Leaderboard => option_env!("DISCORD_LEADERBOARD_WEBHOOK"),
        Channel::Telemetry => option_env!("DISCORD_TELEMETRY_WEBHOOK"),
    } {
        if !url.is_empty() {
            return Some(url);
        }
    }
    None
}

async fn send_message_internal(channel: Channel, msg: String) -> anyhow::Result<()> {
    if let Some(url) = webhook(channel) {
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

pub fn send_message(channel: Channel, msg: String) {
    tokio::spawn(async move {
        if let Err(e) = send_message_internal(channel, msg).await {
            log::warn!("Failed to send Discord message: {}", e);
        }
    });
}
