#![allow(clippy::cast_lossless)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::unreadable_literal)]

#[macro_use]
extern crate err_derive;
#[macro_use]
extern crate itertools;

pub mod video;

#[derive(Debug, Error)]
pub enum MetricsError {
    #[error(display = "Input videos must have matching formats: {}", reason)]
    InputMismatch { reason: &'static str },
    #[error(display = "Input type not supported: {}", reason)]
    UnsupportedInput { reason: &'static str },
    #[error(display = "Could not read input file: {}", reason)]
    MalformedInput { reason: &'static str },
}
