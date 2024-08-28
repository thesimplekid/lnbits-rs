use std::collections::HashMap;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use super::{LNBitsEndpoint, LNBitsRequestKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvoiceResult {
    pub payment_hash: String,
    pub payment_request: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayInvoiceResult {
    pub payment_hash: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateInvoiceParams {
    pub amount: u64,
    pub unit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    /// expiry is in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal: Option<bool>,
    pub out: bool,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecodedInvoice {
    pub payment_hash: String,
    pub amount_msat: i64,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_hash: Option<String>,
    pub payee: String,
    pub date: i64,
    pub expiry: i64,
    pub secret: String,
    pub route_hints: Vec<String>,
    pub min_final_cltv_expiry: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindInvoiceResponse {
    pub checking_id: String,
    pub pending: bool,
    pub amount: i64,
    pub fee: i64,
    pub memo: String,
    pub time: u64,
    pub bolt11: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preimage: Option<String>,
    pub payment_hash: String,
    pub expiry: u64,
    pub extra: HashMap<String, serde_json::Value>,
    pub wallet_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_status: Option<String>,
}

impl crate::LNBitsClient {
    /// Create an invoice
    pub async fn create_invoice(
        &self,
        params: &CreateInvoiceParams,
    ) -> Result<CreateInvoiceResult> {
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
    pub async fn pay_invoice(&self, bolt11: &str) -> Result<PayInvoiceResult> {
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

    pub async fn decode_invoice(&self, invoice: &str) -> Result<DecodedInvoice> {
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

    pub async fn find_invoice(&self, checking_id: &str) -> Result<FindInvoiceResponse> {
        let endpoint = LNBitsEndpoint::PaymentsFindInvoice(checking_id.to_string());

        let body = self.make_get(endpoint, LNBitsRequestKey::Admin).await?;

        match serde_json::from_str(&body) {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Api error response decode invoice");
                log::error!("{}", body);
                bail!("Could not decode invoice")
            }
        }
    }
}
