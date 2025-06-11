//! Payments

use anyhow::Result;
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Deserializer};

use super::LNBitsEndpoint;

/// Payment details
#[derive(Debug, Deserialize)]
pub struct PaymentDetails {
    /// Payment status
    pub status: String,
    /// Checking id
    pub checking_id: String,
    /// Amount
    pub amount: i64,
    /// Fee
    pub fee: i64,
    /// Memo
    pub memo: String,
    /// Time
    #[serde(deserialize_with = "deserialize_time")]
    pub time: String,
    /// Created at
    #[serde(deserialize_with = "deserialize_time")]
    pub created_at: String,
    /// Updated at
    #[serde(deserialize_with = "deserialize_time")]
    pub updated_at: String,
    /// BOLT11
    pub bolt11: String,
    /// Preimage
    pub preimage: Option<String>,
    /// Payment hash
    pub payment_hash: String,
    /// Expiry
    #[serde(deserialize_with = "deserialize_time")]
    pub expiry: String,
    /// Extra
    pub extra: serde_json::Value,
    /// Wallet id
    pub wallet_id: String,
    /// Webhook
    pub webhook: Option<String>,
    /// Webhook status
    pub webhook_status: Option<String>,
    /// Pending
    ///
    /// Pre v1
    pub pending: Option<bool>,
}

/// Custom deserializer for time field that can handle both u64 and string
/// formats
fn deserialize_time<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TimeValue {
        Unix(u64),
        String(String),
    }

    let time_value = TimeValue::deserialize(deserializer)?;

    match time_value {
        TimeValue::Unix(timestamp) => {
            // Convert Unix timestamp (seconds since epoch) to DateTime<Utc>
            let datetime = Utc
                .timestamp_opt(timestamp as i64, 0)
                .single()
                .ok_or_else(|| serde::de::Error::custom("Invalid timestamp"))?;

            // Format the datetime as an ISO 8601 string
            Ok(datetime.to_rfc3339())
        }
        TimeValue::String(s) => Ok(s),
    }
}

/// Payment
#[derive(Debug, Deserialize)]
pub struct Payment {
    /// Paid
    pub paid: bool,
    /// Preimage
    pub preimage: Option<String>,
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
