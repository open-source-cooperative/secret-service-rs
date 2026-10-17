The detailed list of changes in each release can be found at https://github.com/hwchen/secret-service-rs/releases.

## Unreleased

- Use `zbus` 6 and require Rust 1.89.
- Preserve the blocking API with synchronous wrappers around the async API.
- Accept `zbus::Connection` in `blocking::SecretService::connect_with_existing`.
