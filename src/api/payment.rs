//! Payments

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::LNBitsEndpoint;

/// Payment details
#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentDetails {
    /// Payment status
    pub status: String,
    /// Pending
    pub pending: bool,
    /// Checking id
    pub checking_id: String,
    /// Amount
    pub amount: i64,
    /// Fee
    pub fee: i64,
    /// Memo
    pub memo: String,
    /// Time
    pub time: u64,
    /// BOld11
    pub bolt11: String,
    /// Preimage
    pub preimage: String,
    /// Payment hash
    pub payment_hash: String,
    /// Expiry
    pub expiry: u64,
    /// Extra
    pub extra: serde_json::Value,
    /// Wallet id
    pub wallet_id: String,
    /// Webhook
    pub webhook: Option<String>,
    /// Webhook status
    pub webhook_status: Option<String>,
}

/// Payment
#[derive(Debug, Serialize, Deserialize)]
pub struct Payment {
    /// Paid
    pub paid: bool,
    /// Preimage
    pub preimage: String,
    /// Details
    pub details: PaymentDetails,
}

impl crate::LNBitsClient {
    /// Check if invoice paid
    pub async fn get_payment_info(&self, payment_hash: &str) -> Result<Payment> {
        let body = self
            .make_get(
                LNBitsEndpoint::PaymentHash(payment_hash.to_string()),
                crate::api::LNBitsRequestKey::Admin,
            )
            .await?;

        let payment: Payment = serde_json::from_str(&body)?;

        Ok(payment)
    }
}
