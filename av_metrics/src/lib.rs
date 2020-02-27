//! `av_metrics` is a collection of quality metrics for audio and video files.
//! Currently only includes video metrics. Audio metrics will likely be added
//! in the future.

#![allow(clippy::cast_lossless)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::wrong_self_convention)]
#![deny(missing_docs)]

#[macro_use]
extern crate err_derive;
#[macro_use]
extern crate itertools;

pub mod video;

#[cfg(cargo_c)]
mod capi;

#[cfg(cargo_c)]
pub use capi::*;

pub use v_frame;

/// Possible errors that may occur during processing of a metric.
///
/// This enum may be added to in the future and should not be assumed to be exhaustive.
#[derive(Debug, Error)]
pub enum MetricsError {
    /// Indicates an input file could not be read for some reason.
    #[error(display = "Could not read input file: {}", reason)]
    MalformedInput {
        #[doc(hidden)]
        reason: &'static str,
    },
    /// Indicates an input file could be read, but is not supported by the current metric.
    #[error(display = "Input type not supported: {}", reason)]
    UnsupportedInput {
        #[doc(hidden)]
        reason: &'static str,
    },
    /// Indicates two inputs did not have matching formats or resolutions.
    #[error(display = "Input videos must have matching formats: {}", reason)]
    InputMismatch {
        #[doc(hidden)]
        reason: &'static str,
    },
    /// Placeholder
    #[doc(hidden)]
    #[error(display = "Unreachable")]
    NonExhaustive,
}

#[cfg(test)]
#[inline(always)]
fn assert_metric_eq(expected: f64, value: f64) {
    assert!(
        (expected - value).abs() < 0.01,
        "Expected {}, got {}",
        expected,
        value
    );
}
