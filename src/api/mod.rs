//! LNbits api

use std::fmt;

use anyhow::{bail, Result};

pub mod invoice;
pub mod payment;
pub mod wallet;
pub mod webhook;

/// LNbits api key type
#[derive(Debug, Clone, Hash)]
pub enum LNBitsRequestKey {
    /// Admin api key
    Admin,
    /// Read invoice api key
    InvoiceRead,
}

/// LNbits endpoints
#[derive(Debug, Clone, Hash)]
pub enum LNBitsEndpoint {
    /// Payments endpoint
    Payments,
    /// Decode payments endpoint
    PaymentsDecode,
    /// Payments endpoint with hash
    PaymentHash(String),
    /// Find Payements invoice
    PaymentsFindInvoice(String),
    /// Wallet info endpoint
    Wallet,
}

impl fmt::Display for LNBitsEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LNBitsEndpoint::Payments => write!(f, "api/v1/payments"),
            LNBitsEndpoint::PaymentsDecode => write!(f, "api/v1/payments/decode"),
            LNBitsEndpoint::PaymentHash(hash) => write!(f, "api/v1/payments/{}", hash),
            LNBitsEndpoint::Wallet => write!(f, "api/v1/wallet"),
            LNBitsEndpoint::PaymentsFindInvoice(checking_id) => {
                write!(f, "api/v1/payments?checking_id={}", checking_id)
            }
        }
    }
}

impl crate::LNBitsClient {
    /// Make get request
    pub async fn make_get(
        &self,
        endpoint: LNBitsEndpoint,
        key: LNBitsRequestKey,
    ) -> Result<String> {
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
            bail!("Not found")
        }

        let body = response.text().await?;

        Ok(body)
    }

    /// Make post request
    pub async fn make_post(
        &self,
        endpoint: LNBitsEndpoint,
        key: LNBitsRequestKey,
        body: &str,
    ) -> Result<String> {
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
            bail!("Not Found")
        }

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            bail!("Unauthorized")
        }

        let body = response.text().await?;

        Ok(body)
    }
}
