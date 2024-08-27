use serde::{Deserialize, Serialize};

use super::LNBitsEndpoint;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvoiceResult {
    pub payment_hash: String,
    pub payment_request: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayInvoiceResult {
    pub payment_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvoiceParams {
    pub amount: i64,
    pub unit: String,
    pub memo: Option<String>,
    /// expiry is in seconds
    pub expiry: Option<i64>,
    pub webhook: Option<String>,
    pub internal: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecodedInvoice {
    pub payment_hash: String,
    pub amount_msat: i64,
    pub description: String,
    pub description_hash: Option<String>,
    pub payee: String,
    pub date: i64,
    pub expiry: i64,
    pub secret: String,
    pub route_hints: Vec<String>,
    pub min_final_cltv_expiry: i64,
}

impl crate::LNBitsClient {
    /// Create an invoice
    pub async fn create_invoice(
        &self,
        params: &CreateInvoiceParams,
    ) -> Result<CreateInvoiceResult, crate::LNBitsError> {
        // Add out: true to the params
        let params = serde_json::json!({
            "out": false,
            "amount": params.amount,
            "unit": params.unit,
            "memo": params.memo,
            "webhook": params.webhook,
            "internal": params.internal,
            "expiry": params.expiry,
        });

        let body = self
            .make_post(
                LNBitsEndpoint::Payments,
                crate::api::LNBitsRequestKey::InvoiceRead,
                &serde_json::to_string(&params)?,
            )
            .await?;

        let invoice_result: CreateInvoiceResult = serde_json::from_str(&body)?;
        Ok(invoice_result)
    }

    /// Pay an invoice
    pub async fn pay_invoice(&self, bolt11: &str) -> Result<PayInvoiceResult, crate::LNBitsError> {
        let body = self
            .make_post(
                LNBitsEndpoint::Payments,
                crate::api::LNBitsRequestKey::Admin,
                &serde_json::to_string(&serde_json::json!({ "out": true, "bolt11": bolt11 }))?,
            )
            .await?;

        let invoice_result: PayInvoiceResult = serde_json::from_str(&body)?;
        Ok(invoice_result)
    }

    /// Decode bolt11 invoice
    pub async fn decode_invoice(
        &self,
        invoice: &str,
    ) -> Result<DecodedInvoice, crate::LNBitsError> {
        let body = self
            .make_post(
                LNBitsEndpoint::PaymentsDecode,
                crate::api::LNBitsRequestKey::Admin,
                &serde_json::to_string(&serde_json::json!({ "data": invoice }))?,
            )
            .await?;

        let invoice_result: DecodedInvoice = serde_json::from_str(&body)?;
        Ok(invoice_result)
    }

    /// Check if invoice paid
    pub async fn is_invoice_paid(&self, payment_hash: &str) -> Result<bool, crate::LNBitsError> {
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
