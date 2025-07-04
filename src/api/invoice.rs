//! Invoice related api

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use super::LNBitsEndpoint;

/// Create invoice response
#[derive(Debug, Deserialize)]
pub struct CreateInvoiceResponse {
    /// Payment hash
    payment_hash: String,
    /// Bolt11
    bolt11: Option<String>,
    /// Payment request (PRE v1)
    payment_request: Option<String>,
}

impl CreateInvoiceResponse {
    /// Payment hash
    pub fn payment_hash(&self) -> &str {
        &self.payment_hash
    }

    /// Get bolt11 from response
    pub fn bolt11(&self) -> Option<String> {
        self.bolt11.clone().or_else(|| self.payment_request.clone())
    }
}

/// Pay invoice response
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize)]
pub struct PayInvoiceResponse {
    /// Payment hash
    pub payment_hash: String,
}

/// Create invoice request
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateInvoiceRequest {
    /// Amount (sat)
    pub amount: u64,
    /// Unit
    pub unit: String,
    /// Memo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    /// Expiry is in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<u64>,
    /// Webhook url
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Internal payment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal: Option<bool>,
    /// Incoming or outgoing payment
    pub out: bool,
}

/// Decode invoice response
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DecodeInvoiceResponse {
    /// Payment hash
    pub payment_hash: String,
    /// Amount (msat)
    pub amount_msat: i64,
    /// Description
    pub description: String,
    /// Description hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_hash: Option<String>,
    /// Payee
    pub payee: String,
    /// Date
    pub date: i64,
    /// Expiry
    pub expiry: f64,
    /// Secret
    pub secret: String,
    /// Route hints
    pub route_hints: Vec<String>,
    /// Mint final cltx expiry
    pub min_final_cltv_expiry: i64,
}

impl crate::LNBitsClient {
    /// Create an invoice
    pub async fn create_invoice(
        &self,
        params: &CreateInvoiceRequest,
    ) -> Result<CreateInvoiceResponse> {
        let body = self
            .make_post(
                LNBitsEndpoint::Payments,
                crate::api::LNBitsRequestKey::InvoiceRead,
                &serde_json::to_string(&params)?,
            )
            .await?;

        match serde_json::from_str(&body) {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Api error response on invoice creation");
                log::error!("{}", body);
                bail!("Could not create invoice")
            }
        }
    }

    /// Pay an invoice
    pub async fn pay_invoice(
        &self,
        bolt11: &str,
        _amount_sats: Option<u64>,
    ) -> Result<PayInvoiceResponse> {
        let body = self
            .make_post(
                LNBitsEndpoint::Payments,
                crate::api::LNBitsRequestKey::Admin,
                &serde_json::to_string(&serde_json::json!({ "out": true, "bolt11": bolt11 }))?,
            )
            .await?;

        match serde_json::from_str(&body) {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Api error response on paying invoice");
                log::error!("{}", body);
                bail!("Could not pay invoice")
            }
        }
    }

    /// Decode invoice
    pub async fn decode_invoice(&self, invoice: &str) -> Result<DecodeInvoiceResponse> {
        let body = self
            .make_post(
                LNBitsEndpoint::PaymentsDecode,
                crate::api::LNBitsRequestKey::Admin,
                &serde_json::to_string(&serde_json::json!({ "data": invoice }))?,
            )
            .await?;

        match serde_json::from_str(&body) {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Api error response decode invoice");
                log::error!("{}", body);
                bail!("Could not decode invoice")
            }
        }
    }

    /// Check if invoice paid
    pub async fn is_invoice_paid(&self, payment_hash: &str) -> Result<bool> {
        let body = self
            .make_get(
                LNBitsEndpoint::PaymentHash(payment_hash.to_string()),
                crate::api::LNBitsRequestKey::Admin,
            )
            .await?;

        let invoice_result: serde_json::Value = serde_json::from_str(&body)?;
        Ok(invoice_result["paid"].as_bool().unwrap_or(false))
    }
}
