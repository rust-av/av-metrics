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
av-metrics = "0.3"
```

Then check out [the API docs](https://docs.rs/av-metrics/).

### As a binary

#### Windows

Downlad the latest binary from the [Releases](https://github.com/rust-av/av-metrics/releases) tab.

#### OS X and Linux

Pre-built binaries are coming soon. In the meantime, have the latest stable Rust
installed and run the following:

```
cargo install av-metrics-tool
```

#### Usage

From any terminal, run the executable with your two video files as arguments:

```
➜ av-metrics-tool lossless.y4m lossy.y4m
```

You should receive output for all supported metrics:

```
PSNR - Y: 32.5281  U: 36.4083  V: 39.8238  Avg: 33.6861
APSNR - Y: 32.5450  U: 36.4087  V: 39.8244  Avg: 33.6995
PSNR HVS - Y: 34.3225  U: 37.7400  V: 40.5569  Avg: 31.8674
SSIM - Y: 13.2572  U: 10.8624  V: 12.8369  Avg: 12.6899
MSSSIM - Y: 18.8343  U: 16.6943  V: 18.7662  Avg: 18.3859
CIEDE2000 - 36.2820
```

Currently, the tool can only decode y4m files. Both files must match in resolution, bit depth, and color sampling.
