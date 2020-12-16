//! This crate providers ready-made decoders for use with av-metrics or other tools as needed.
//!
//! No decoders are enabled by default. They must be enabled via Cargo features.
//!
//! Currently supported decoder features: y4m

#![deny(missing_docs)]

#[cfg(feature = "y4m")]
mod y4m;

#[cfg(feature = "y4m")]
pub use crate::y4m::Y4MDecoder;
