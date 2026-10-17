# Secret Service

[![crates.io version](https://img.shields.io/crates/v/secret-service.svg)](https://crates.io/crates/secret-service)
[![crate documentation](https://docs.rs/secret-service/badge.svg)](https://docs.rs/secret-service)
![MSRV](https://img.shields.io/badge/rustc-1.89+-blue.svg)
[![crates.io downloads](https://img.shields.io/crates/d/secret-service.svg)](https://crates.io/crates/secret-service)
![CI](https://github.com/hwchen/secret-service-rs/workflows/CI/badge.svg) [![codecov](https://codecov.io/gh/open-source-cooperative/secret-service-rs/graph/badge.svg)](https://codecov.io/gh/open-source-cooperative/secret-service-rs)

A rust library for interacting with the FreeDesktop Secret Service API through DBus.

### Basic Usage

`secret-service` is implemented in pure Rust by default, so it doesn't require any system libraries
such as `libdbus-1-dev` or `libdbus-1-3` on Ubuntu.

**In Cargo.toml:**

When adding the crate, you must select a feature representing your selected cryptography backend (either `crypto-rust` or `crypto-openssl`). For example:

```toml
[dependencies]
secret-service = { version = "6", features = ["crypto-rust"] }
```

If your cargo dependency tree does not already specify the asynchronous runtime to be used by the `zbus` crate, you must select a feature corresponding to your selected runtime (either `rt-tokio` or `rt-async-io`). For example:

```toml
[dependencies]
secret-service = { version = "6", features = ["crypto-rust", "rt-tokio"]}
```

For convenience, the following four combined feature flags are available that select both runtime and cryptography:

- `rt-async-io-crypto-rust`: Async I/O runtime with pure rust cryptography.
- `rt-async-io-crypto-openssl`: Async I/O runtime with OpenSSL cryptography.
- `rt-tokio-crypto-rust`: Tokio runtime and pure Rust cryptography.
- `rt-tokio-crypto-openssl`: Tokio runtime and OpenSSL cryptography.

Note that using OpenSSL cryptography requires the OpenSSL development libraries be installed on your build system. If they are not, you can activate the `bundled` feature of the `openssl` crate in your cargo dependencies to statically link OpenSSL cryptography into your application.

**In source code:** 

This example is an application built with the Tokio runtime.

```rust
use secret_service::SecretService;
use secret_service::EncryptionType;
use std::{collections::HashMap, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // initialize secret service (dbus connection and encryption session)
    let ss = SecretService::connect(EncryptionType::Dh).await?;

    // get default collection
    let collection = ss.get_default_collection().await?;

    // create new item
    collection.create_item(
        "test_label", // label
        HashMap::from([("test", "test_value")]), // properties
        b"test_secret", // secret
        false, // replace item with same attributes
        "text/plain" // secret content type
    ).await?;

    // search items by properties
    let search_items = ss.search_items(
        HashMap::from([("test", "test_value")])
    ).await?;

    let item = search_items.unlocked.first().ok_or("Not found!")?;

    // retrieve secret from item
    let secret = item.get_secret().await?;
    assert_eq!(secret, b"test_secret");

    // delete item (deletes the dbus object, not the struct instance)
    item.delete().await?;
    Ok(())
}
```

### Functionality

- SecretService: initialize dbus or use existing connection, create plain/encrypted session.
- Collections: create, delete, search, get-by-path.
- Items: create, delete, search, get-by-path, get/set secret.

### Migrating from 5

Version 6 uses `zbus` 6 and requires Rust 1.89. It removes the `blocking` module, following
[zbus's removal of its blocking API](https://github.com/z-galaxy/zbus/pull/1976). Call the async API inside
`zbus::block_on` if your application does not use an async runtime:

```rust
let label = zbus::block_on(async {
    let service = secret_service::SecretService::connect(secret_service::EncryptionType::Dh).await?;
    let collection = service.get_default_collection().await?;
    collection.get_label().await
})?;
```

### Changelog
See [the list of GitHub releases and their release notes](https://github.com/hwchen/secret-service-rs/releases).

### Versioning
This library is feature complete and has stabilized its API for the most part. However, as this
crate is almost soley reliable on the `zbus` crate, we try and match major version releases
with theirs to handle breaking changes and move with the wider `zbus` ecosystem.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
