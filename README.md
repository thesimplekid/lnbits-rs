# lnbits-rs


An ergonomic, [LNbits](https://lnbits.com/) API Client for Rust.



## Key features

- [x] Create invoices
- [x] Webhook for incoming invoices
- [x] Decode invoices
- [x] Pay invoices
- [x] Get wallet details
- [x] Tor support

## Minimum Supported Rust Version (MSRV)

The `phoenixd` library should always compile with any combination of features on Rust **1.63.0**.

To build and test with the MSRV you will need to pin the below dependency versions:

```shell
cargo update -p tokio --precise 1.38.1
cargo update -p reqwest --precise 0.12.4
```

## License

Licensed under MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
