//! `av_metrics` is a collection of quality metrics for audio and video files.
//! Currently only includes video metrics. Audio metrics will likely be added
//! in the future.

#![allow(clippy::cast_lossless)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::wrong_self_convention)]
#![deny(missing_docs)]

#[macro_use]
extern crate itertools;
#[macro_use]
extern crate thiserror;

pub mod video;

/// Possible errors that may occur during processing of a metric.
///
/// This enum may be added to in the future and should not be assumed to be exhaustive.
#[derive(Debug, Error)]
pub enum MetricsError {
    /// Indicates an input file could not be read for some reason.
    #[error("Could not read input file: {reason}")]
    MalformedInput {
        #[doc(hidden)]
        reason: &'static str,
    },
    /// Indicates an input file could be read, but is not supported by the current metric.
    #[error("Input type not supported: {reason}")]
    UnsupportedInput {
        #[doc(hidden)]
        reason: &'static str,
    },
    /// Indicates two inputs did not have matching formats or resolutions.
    #[error("Input videos must have matching formats: {reason}")]
    InputMismatch {
        #[doc(hidden)]
        reason: &'static str,
    },
    /// Indicates the impossibility to process the two videos.
    #[error("Could not process the two videos: {reason}")]
    VideoError {
        #[doc(hidden)]
        reason: String,
    },
    /// Indicates the impossibility to send two frames in order to be processed.
    #[error("Could not send two frames to be processed: {reason}")]
    SendError {
        #[doc(hidden)]
        reason: String,
    },
    /// Indicates the impossibility to process two frames.
    #[error("Could not process two frames: {reason}")]
    ProcessError {
        #[doc(hidden)]
        reason: String,
    },
    /// Placeholder
    #[doc(hidden)]
    #[error("Unreachable")]
    NonExhaustive,
}
