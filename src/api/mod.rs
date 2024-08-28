use std::fmt;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

pub mod invoice;
pub mod wallet;
pub mod webhook;

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum LNBitsRequestKey {
    Admin,
    InvoiceRead,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub enum LNBitsEndpoint {
    Payments,
    PaymentsDecode,
    PaymentHash(String),
    PaymentsFindInvoice(String),
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
