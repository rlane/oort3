use std::env;
use tokio::sync::mpsc;

use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use tokio::sync::Mutex;

fn channel_id() -> ChannelId {
    match std::env::var("ENVIRONMENT") {
        Ok(x) if x == "dev" => ChannelId(1044848260893900862),
        Ok(x) if x == "prod" => ChannelId(1045042156060016680),
        _ => {
            panic!("Invalid ENVIRONMENT")
        }
    }
}

#[derive(Clone, Debug)]
pub struct Msg {
    pub text: String,
}

struct Handler {
    rx: Mutex<Option<mpsc::Receiver<Msg>>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        log::info!("Connected to Discord as {}", ready.user.name);

        if let Some(mut rx) = self.rx.lock().await.take() {
            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    log::info!("Sending Discord message {:?}", msg.text);
                    if let Err(e) = channel_id().say(&ctx.http, &msg.text).await {
                        log::error!("Error sending message: {:?}", e);
                    }
                }
            });
        } else {
            panic!("RX channel already taken");
        }
    }
}

pub async fn start() -> Result<mpsc::Sender<Msg>, anyhow::Error> {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let (tx, rx) = mpsc::channel(10);

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            rx: Mutex::new(Some(rx)),
        })
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    tokio::spawn(async move {
        if let Err(why) = client.start().await {
            log::error!("Discord client error: {:?}", why);
        }
    });

    Ok(tx)
}
