use crate::video::pixel::Pixel;
use crate::video::FrameInfo;

#[cfg(feature = "y4m-decode")]
mod y4m;

#[cfg(feature = "y4m-decode")]
pub use self::y4m::*;

/// A trait for allowing metrics to decode generic video formats.
///
/// Currently, y4m decoding support using the `y4m` crate is built-in
/// to this crate. This trait is extensible so users may implement
/// their own decoders.
pub trait Decoder<T: Pixel> {
    /// Read the next frame from the input video.
    ///
    /// Expected to return `Err` if the end of the video is reached.
    fn read_video_frame(&mut self) -> Result<FrameInfo<T>, ()>;
}
