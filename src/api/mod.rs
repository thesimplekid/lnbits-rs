use std::fmt;

use serde::{Deserialize, Serialize};

pub mod invoice;
pub mod paid_invoice_stream;
pub mod wallet;

#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum LNBitsRequestKey {
    Admin,
    InvoiceRead,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum LNBitsEndpoint {
    Payments,
    PaymentsDecode,
    PaymentHash(String),
    Wallet,
}

impl fmt::Display for LNBitsEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LNBitsEndpoint::Payments => write!(f, "api/v1/payments"),
            LNBitsEndpoint::PaymentsDecode => write!(f, "api/v1/payments/decode"),
            LNBitsEndpoint::PaymentHash(hash) => write!(f, "api/v1/payments/{}", hash),
            LNBitsEndpoint::Wallet => write!(f, "api/v1/wallet"),
        }
    }
}

impl crate::LNBitsClient {
    pub async fn make_get(
        &self,
        endpoint: LNBitsEndpoint,
        key: LNBitsRequestKey,
    ) -> Result<String, crate::LNBitsError> {
        let url = self.lnbits_url.join(&endpoint.to_string())?;
        let response = self
            .reqwest_client
            .get(url)
            .header("X-Api-Key", {
                match key {
                    LNBitsRequestKey::Admin => self.admin_key.clone(),
                    LNBitsRequestKey::InvoiceRead => self.invoice_read_key.clone(),
                }
            })
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(crate::LNBitsError::NotFound);
        }

        let body = response.text().await?;

        Ok(body)
    }

    pub async fn make_post(
        &self,
        endpoint: LNBitsEndpoint,
        key: LNBitsRequestKey,
        body: &str,
    ) -> Result<String, crate::LNBitsError> {
        let url = self.lnbits_url.join(&endpoint.to_string())?;
        let response = self
            .reqwest_client
            .post(url)
            .header("X-Api-Key", {
                match key {
                    LNBitsRequestKey::Admin => self.admin_key.clone(),
                    LNBitsRequestKey::InvoiceRead => self.invoice_read_key.clone(),
                }
            })
            .body(body.to_string())
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(crate::LNBitsError::NotFound);
        }

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(crate::LNBitsError::Unauthorized);
        }

        let body = response.text().await?;

        Ok(body)
    }
}
