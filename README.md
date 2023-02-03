# Tokio Shutdown

[![Build Status][build-img]][build-url]
[![Repository][crates-img]][crates-url]
[![Documentation][doc-img]][doc-url]

[build-img]: https://img.shields.io/github/actions/workflow/status/dnaka91/tokio-shutdown/ci.yml?branch=main&style=for-the-badge
[build-url]: https://github.com/dnaka91/tokio-shutdown/actions?query=workflow%3ACI
[crates-img]: https://img.shields.io/crates/v/tokio-shutdown?style=for-the-badge
[crates-url]: https://crates.io/crates/tokio-shutdown
[doc-img]: https://img.shields.io/badge/docs.rs-tokio--shutdown-4d76ae?style=for-the-badge
[doc-url]: https://docs.rs/tokio-shutdown

Tiny crate that allows to wait for a stop signal across multiple threads. Helpful mostly in server
applications that run indefinitely and need a signal for graceful shutdowns.

## Usage

Add `tokio-shutdown` to your project with `cargo add tokio-shutdown` (needs [cargo-edit]) or add it
manually to your `Cargo.toml`:

```toml
[dependencies]
tokio-shutdown = "<latest-version>"
```

In addition, you will need to use the lastest [tokio](https://tokio.rs) runtime to use this library,
as it uses async/await and is bound to this runtime.

[cargo-edit]: https://github.com/killercup/cargo-edit

### Example

For examples check out the [basic](examples/basic.rs) and [streaming](examples/streaming.rs), or
consult the [docs](doc-url).

## License

This project is licensed under [MIT License](LICENSE) (or <http://opensource.org/licenses/MIT>).
