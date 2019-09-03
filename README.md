# Quality metrics

[![crate](https://img.shields.io/crates/v/av-metrics.svg)](https://crates.io/crates/av-metrics)
[![docs](https://docs.rs/av-metrics/badge.svg)](https://docs.rs/av-metrics/)
[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Actions Status](https://github.com/rust-av/av-metrics/workflows/ci/badge.svg)](https://github.com/rust-av/av-metrics/actions)
[![dependency status](https://deps.rs/repo/github/rust-av/av-metrics/status.svg)](https://deps.rs/repo/github/rust-av/av-metrics)
[![IRC](https://img.shields.io/badge/irc-%23rust--av-blue.svg)](http://webchat.freenode.net?channels=%23rust-av&uio=d4)

## Video Metrics implemented

 - [X] PSNR
 - [X] APSNR
 - [X] PSNR HVS
 - [X] SSIM
 - [X] MSSSIM
 - [X] CIEDE2000

## Installation

### As a library

Add the following to your Cargo.toml
```toml
av-metrics = "0.2"
```

Then check out [the API docs](https://docs.rs/av-metrics/).

### As a binary

Pre-built binaries are coming soon. In the meantime, have the latest stable Rust
installed and run the following:

```
cargo install av-metrics-tool
```
