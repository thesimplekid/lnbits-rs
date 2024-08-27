use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
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

async fn handle_invoice(
    State(_state): State<WebhookState>,
    Json(payload): Json<Value>,
) -> Result<StatusCode, StatusCode> {
    let webhook_response: WebhookResponse =
        serde_json::from_value(payload.clone()).map_err(|_err| {
            log::warn!("Got an invalid payload on webhook");
            log::debug!("Value: {}", payload);

            StatusCode::UNPROCESSABLE_ENTITY
        })?;

    log::debug!("Received webhook update for: {}", webhook_response.data);

    Ok(StatusCode::OK)
}

/// Webhook response
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookResponse {
    /// Webhook id
    pub id: String,
    /// Webhook url
    pub data: String,
    /// Webhook Version
    pub webhook_version: String,
    /// Enabled
    pub enabled: bool,
    /// Event types
    pub event_types: Vec<String>,
}
