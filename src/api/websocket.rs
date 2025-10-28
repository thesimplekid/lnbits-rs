//! Websocket

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

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
    pub async fn subscribe_to_websocket(&self) -> anyhow::Result<()> {
        // Create a new channel for this connection
        // This ensures old receivers will get None and new receivers will work
        let (new_sender, new_receiver) = tokio::sync::mpsc::channel(8);

        // Replace the sender and receiver with the new ones
        *self.sender.lock().await = new_sender;
        *self.receiver.lock().await = new_receiver;

        let base_url = self
            .lnbits_url
            .to_string()
            .trim_end_matches('/')
            .replace("http", "ws");
        let ws_url = format!("{}/api/v1/ws/{}", base_url, self.invoice_read_key);

        let (ws_stream, _) = connect_async(ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        let sender = self.sender.clone();

        // Handle incoming messages with timeout detection
        tokio::spawn(async move {
            let mut last_message_time = std::time::Instant::now();
            let timeout_duration = std::time::Duration::from_secs(60); // 60 second timeout

            loop {
                // Use timeout to detect dead connections
                let message_result =
                    tokio::time::timeout(std::time::Duration::from_secs(30), read.next()).await;

                match message_result {
                    Ok(Some(message)) => {
                        last_message_time = std::time::Instant::now();
                        match message {
                            Ok(msg) => {
                                match msg {
                                    Message::Text(text) => {
                                        tracing::trace!("Received websocket message: {}", text);

                                        // Parse the message
                                        if let Ok(message) =
                                            serde_json::from_str::<WebSocketMessage>(&text)
                                        {
                                            if let Some(payment) = message.payment {
                                                if payment.amount > 0 {
                                                    tracing::info!(
                                                        "Payment received: {}",
                                                        payment.payment_hash
                                                    );
                                                    let sender = sender.lock().await;
                                                    if let Err(err) =
                                                        sender.send(payment.payment_hash).await
                                                    {
                                                        log::error!(
                                                            "Failed to send payment hash: {}",
                                                            err
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Message::Ping(_) | Message::Pong(_) => {
                                        // Keepalive messages
                                        tracing::trace!("Received ping/pong");
                                    }
                                    Message::Close(_) => {
                                        tracing::warn!("WebSocket closed by server");
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                            Err(e) => {
                                tracing::error!("Error receiving websocket message: {}", e);
                                break;
                            }
                        }
                    }
                    Ok(None) => {
                        // Stream ended
                        tracing::warn!("WebSocket stream ended");
                        break;
                    }
                    Err(_) => {
                        // Timeout - check if we've exceeded the overall timeout
                        if last_message_time.elapsed() > timeout_duration {
                            tracing::warn!(
                                "WebSocket timeout - no messages received for {:?}",
                                timeout_duration
                            );
                            break;
                        }
                        // Send a ping to keep connection alive and detect dead connections
                        if let Err(e) = write.send(Message::Ping(vec![].into())).await {
                            tracing::error!("Failed to send ping: {}", e);
                            break;
                        }
                        tracing::trace!("Sent ping to keep connection alive");
                    }
                }
            }

            tracing::info!("WebSocket task ending, sender will be dropped");
            // Task ends, sender gets dropped, receiver will get None
        });

        Ok(())
    }
}
