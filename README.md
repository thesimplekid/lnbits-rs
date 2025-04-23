# lnbits-rs

A comprehensive Rust client for the [LNbits](https://lnbits.com/) API. LNbits is a free and open-source lightning wallet and account system.

## Key features

- [x] Create invoices
- [x] Webhook for incoming invoices
- [x] Decode invoices
- [x] Pay invoices
- [x] Get wallet details
- [x] Tor support

## Usage

```rust
use lnbits_rs::LNBitsClient;

async fn example() -> anyhow::Result<()> {
    // Create a new client
    let client = LNBitsClient::new(
        "wallet_id",
        "admin_key",
        "invoice_read_key",
        "https://legend.lnbits.com",
        None, // Optional Tor socket
    )?;
    
    // Get wallet details
    let wallet = client.get_wallet_details().await?;
    println!("Wallet balance: {} sats", wallet.balance);
    
    Ok(())
}
```

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
