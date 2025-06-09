//! Webhook

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;

use crate::LNBitsClient;

/// Webhook state
#[derive(Debug, Clone)]
pub struct WebhookState {
    /// Sender
    pub sender: tokio::sync::mpsc::Sender<String>,
}

impl LNBitsClient {
    /// Create invoice webhook
    pub async fn create_invoice_webhook_router(
        &self,
        webhook_endpoint: &str,
        sender: tokio::sync::mpsc::Sender<String>,
    ) -> anyhow::Result<Router> {
        let state = WebhookState { sender };

        let router = Router::new()
            .route(webhook_endpoint, post(handle_invoice))
            .with_state(state);

        Ok(router)
    }
}

/// Webhook request
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WebhookRequest {
    /// Checking id
    pub checking_id: String,
}

async fn handle_invoice(
    State(state): State<WebhookState>,
    Json(payload): Json<Value>,
) -> Result<StatusCode, StatusCode> {
    let webhook_response: WebhookRequest =
        serde_json::from_value(payload.clone()).map_err(|_err| {
            log::warn!("Got an invalid payload on webhook");
            log::debug!("Value: {}", payload);

            StatusCode::UNPROCESSABLE_ENTITY
        })?;

    log::debug!(
        "Received webhook update for: {}",
        webhook_response.checking_id
    );

    if let Err(err) = state.sender.send(webhook_response.checking_id).await {
        log::warn!("Could not send on channel: {}", err);
    }

    Ok(StatusCode::OK)
}
