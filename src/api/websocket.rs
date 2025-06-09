//! Websocket

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::LNBitsClient;

#[derive(Debug, Deserialize)]
struct WebSocketPayment {
    payment_hash: String,
    amount: i64,
}

#[derive(Debug, Deserialize)]
struct WebSocketMessage {
    payment: Option<WebSocketPayment>,
}

impl LNBitsClient {
    /// Subscribe to websocket updates
    pub async fn subscribe_to_websocket(
        &self,
        sender: tokio::sync::mpsc::Sender<String>,
    ) -> anyhow::Result<()> {
        let ws_url = format!(
            "{}/api/v1/ws/{}",
            self.lnbits_url.to_string().replace("http", "ws"),
            self.admin_key
        );

        let (ws_stream, _) = connect_async(ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Subscribe to updates
        write.send(Message::Text("subscribe".into())).await?;

        // Handle incoming messages
        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(msg) => {
                        if let Message::Text(text) = msg {
                            tracing::debug!("Received websocket message: {}", text);

                            // Parse the message
                            if let Ok(message) = serde_json::from_str::<WebSocketMessage>(&text) {
                                if let Some(payment) = message.payment {
                                    if payment.amount > 0 {
                                        tracing::info!(
                                            "Payment received: {}",
                                            payment.payment_hash
                                        );
                                        if let Err(err) = sender.send(payment.payment_hash).await {
                                            log::error!("Failed to send payment hash: {}", err);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error receiving websocket message: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}
