#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]

use anyhow::Result;
use reqwest::Url;

pub mod api;

#[derive(Clone)]
/// The LNBitsClient struct
pub struct LNBitsClient {
    // wallet_id: String, // Can be used later
    admin_key: String,
    invoice_read_key: String,
    lnbits_url: Url,
    // tor_socket: Option<String>, // Can be used later
    reqwest_client: reqwest::Client,
}

impl LNBitsClient {
    /// Create a new LNBitsClient
    pub fn new(
        // Can be used later
        _wallet_id: &str,
        admin_key: &str,
        invoice_read_key: &str,
        lnbits_url: &str,
        tor_socket: Option<&str>,
    ) -> Result<LNBitsClient> {
        let lnbits_url = Url::parse(lnbits_url)?;

        let client = {
            if let Some(tor_socket) = tor_socket {
                let proxy = reqwest::Proxy::all(tor_socket).expect("tor proxy should be there");
                reqwest::Client::builder().proxy(proxy).build()?
            } else {
                reqwest::Client::builder().build()?
            }
        };

        Ok(LNBitsClient {
            // wallet_id,
            admin_key: admin_key.to_string(),
            invoice_read_key: invoice_read_key.to_string(),
            lnbits_url,
            // tor_socket,
            reqwest_client: client,
        })
    }
}
