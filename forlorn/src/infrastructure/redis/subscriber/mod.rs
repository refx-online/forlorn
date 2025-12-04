use anyhow::Result;
use tokio_stream::StreamExt;

use crate::state::AppState;

mod refresh_map;

const CHANNELS: &[&str] = &["forlorn:refresh_map"];

pub struct SubscriberHandler {
    state: AppState,
}

impl SubscriberHandler {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn start_listener(&self) -> Result<()> {
        let mut pubsub = self.state.subscriber.lock().await;

        for ch in CHANNELS {
            pubsub.subscribe(*ch).await?;
            tracing::info!("subscribed to {}!", ch);
        }

        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            let channel = msg.get_channel_name().to_string();
            let payload: String = msg.get_payload()?;
            let state = self.state.clone();

            tokio::spawn(async move {
                let result = match channel.as_str() {
                    "forlorn:refresh_map" => refresh_map::refresh_map(&state.db, &payload).await,

                    _ => Ok(()),
                };

                if let Err(e) = result {
                    tracing::error!("{channel} failed: {e:?}");
                }
            });
        }

        Ok(())
    }
}
