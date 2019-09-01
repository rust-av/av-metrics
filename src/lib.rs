#![allow(clippy::cast_lossless)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::unreadable_literal)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate itertools;

pub mod video;

#[derive(Debug, Fail)]
pub enum MetricsError {
    #[fail(display = "{}", reason)]
    InputMismatch { reason: &'static str },
    #[fail(display = "{}", reason)]
    UnsupportedInput { reason: &'static str },
}
