use std::pin::Pin;
use std::time::Duration;

use futures::stream::unfold;
use futures::{Stream, StreamExt};
use reqwest::header::{HeaderMap, ACCEPT, CACHE_CONTROL, CONNECTION};
use serde_json::Value;
use tokio::time::sleep;

use crate::{LNBitsClient, LNBitsError};

impl LNBitsClient {
    pub async fn paid_invoice(
        &self,
    ) -> Result<Pin<Box<dyn Stream<Item = String> + Send>>, LNBitsError> {
        let url = self.lnbits_url.join("/api/v1/payments/sse")?;

        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            "text/event-stream"
                .parse()
                .map_err(|_| LNBitsError::InvalidHeader)?,
        );
        headers.insert(
            CACHE_CONTROL,
            "no-cache".parse().map_err(|_| LNBitsError::InvalidHeader)?,
        );
        headers.insert(
            CONNECTION,
            "keep-alive"
                .parse()
                .map_err(|_| LNBitsError::InvalidHeader)?,
        );

        let request = self.reqwest_client.get(url).headers(headers).send().await;

        let response = match request {
            Ok(r) => r,
            Err(_) => {
                return Err(LNBitsError::InvoiceStream);
            }
        };

        let stream = response.bytes_stream();

        // The initial state for the unfold function.
        let initial_state = (stream, false); // (stream, sse_trigger)

        Ok(
            unfold(initial_state, |(mut stream, mut sse_trigger)| async move {
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(data) => {
                            let line = String::from_utf8_lossy(&data);

                            if line.starts_with("event: payment-received") {
                                sse_trigger = true;
                                continue;
                            } else if sse_trigger && line.starts_with("data:") {
                                if let Some(json_data) = line.strip_prefix("data:") {
                                    if let Ok(parsed) = serde_json::from_str::<Value>(json_data) {
                                        if let Some(payment_hash) = parsed.get("payment_hash") {
                                            if let Some(hash_str) = payment_hash.as_str() {
                                                // Yield the hash and the updated state.
                                                return Some((
                                                    hash_str.to_string(),
                                                    (stream, false),
                                                ));
                                            }
                                        }
                                    }
                                }
                                sse_trigger = false;
                            } else {
                                sse_trigger = false;
                            }
                        }
                        Err(_) => {
                            // Error reading data, return None to terminate the stream.
                            return None;
                        }
                    }
                }
                // Sleep for 1 second if the stream ends without errors.
                sleep(Duration::from_secs(1)).await;
                None // Indicate the end of the stream.
            })
            .boxed(),
        )
    }
}
