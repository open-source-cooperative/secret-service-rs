# Secret Service

[![crates.io version](https://img.shields.io/crates/v/secret-service.svg)](https://crates.io/crates/secret-service)
[![crate documentation](https://docs.rs/secret-service/badge.svg)](https://docs.rs/secret-service)
![MSRV](https://img.shields.io/badge/rustc-1.89+-blue.svg)
[![crates.io downloads](https://img.shields.io/crates/d/secret-service.svg)](https://crates.io/crates/secret-service)
![CI](https://github.com/hwchen/secret-service-rs/workflows/CI/badge.svg) [![codecov](https://codecov.io/gh/open-source-cooperative/secret-service-rs/graph/badge.svg)](https://codecov.io/gh/open-source-cooperative/secret-service-rs)

A Rust library for interacting with the FreeDesktop Secret Service API through the DBus.

### Basic Usage

When specifying this crate as a dependency, you must select a feature representing your selected cryptography backend (either `crypto-rust` or `crypto-openssl`). For example:

```toml
[dependencies]
secret-service = { version = "6", features = ["crypto-rust"] }
```

Using OpenSSL cryptography requires the OpenSSL development libraries be installed on your build system. If they are not, you can activate the `bundled` feature of the `openssl` crate in your cargo dependencies to statically link OpenSSL cryptography into your application.

This crate supports both blocking and async APIs for accessing the Secret Service. The funtionality provided by both APIs is the same, and includes:

- SecretService: initialize dbus or use existing connection, create plain or encrypted session (the use of encryption is highly recommended).
- Collections: create, delete, search, get-by-path.
- Items: create, delete, search, get-by-path, get/set secret.

The crate comes with two example programs, named `async` and `blocking`, that demonstrate how to use these APIs.

### Advanced Usage

Because this crate is built on top of the [`zbus`](https://crates.io/crates/zbus) crate, it supports two additional features, `rt-default` and `rt-tokio`, which map to the `default-rt` and `tokio` features, respectively, of the `zbus` crate. These features need only be used if your application has specialized async runtime requirements.

- This crate’s `rt-default` feature passes the `default-rt` feature to this crate’s `zbus` dependency. Note that `rt-default` is a _default_ feature of this crate, so if you don’t want `zbus` to be built with the `default-rt` feature then you must suppress the default features of this crate in your dependencies.
- This crate’s `rt-tokio` feature passes the `tokio` feature to this crate’s `zbus` dependency. This is _not_ a default feature of this crate, and its not even required if your application is built using the `tokio` runtime. You only need this feature if you use the blocking API of this crate and you want to make sure blocking calls use `tokio` executor rather than the `zbus` default executor.

The `rt-default` and `rt-tokio` features of this crate are _not_ mutually exclusive, just as their underlying features are not mutually exclusive in the `zbus` crate. See the [`zbus` docs](https://docs.rs/zbus/latest/zbus/) for more information about these features.

Note: This crate always specifies the `proxy` feature (and no other default features) in its `zbus` dependency; there is no way to suppress this.

### Versioning

Since this crate is feature-complete, new major versions are not expected. However, because this crate is built on top of the [`zbus`](https://crates.io/crates/zbus) crate, and that crate is under active development, a new major version of this crate is typically released whenever a new major version of the `zbus` is released. When that happens, the major version number of this crate is matched to the major version number of the `zbus` crate it’s based on.

### Migration from Version 5

Version 6 of this crate uses `zbus` version 6 and requires Rust 1.89.

Because there have been big changes in the `zbus` feature set going from version 5 to version 6, this crate has made similar changes. If you were using any of the the `rt-async-io` features in your version 5 code, they are no longer needed, just specify your encryption feature and you’re good to go. If you were using any of the `rt-tokio` features in your version 5 code, and you don’t use the blocking API, then you can just drop the runtime feature completely and specify your encryption. If you want to use the blocking API and be sure it’s running on your existing `tokio` executor, then specify the `rt-tokio` feature to this crate.

The `blocking::SecretService::connect_with_existing` call now accepts a standard
`zbus::Connection` because the `zbus::blocking::Connection` type has gone away in version 6.

### Changelog

See [the list of GitHub releases and their release notes](https://github.com/hwchen/secret-service-rs/releases).

### License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.