use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use super::LNBitsEndpoint;

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletDetails {
    /// Documented as "id" in the API docs, but is actually not sent
    pub id: Option<String>,
    pub name: String,
    pub balance: i64,
}

impl crate::LNBitsClient {
    pub async fn get_wallet_details(&self) -> Result<WalletDetails> {
        let body = self
            .make_get(
                LNBitsEndpoint::Wallet,
                crate::api::LNBitsRequestKey::InvoiceRead,
            )
            .await?;
        match serde_json::from_str(&body) {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Api error response on get wallet details");
                log::error!("{}", body);
                bail!("Could not get wallet details")
            }
        }
    }
}
