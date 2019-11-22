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
pub trait Decoder: Send {
    /// Read the next frame from the input video.
    ///
    /// Expected to return `Err` if the end of the video is reached.
    fn read_video_frame<T: Pixel>(&mut self) -> Result<FrameInfo<T>, ()>;
    /// Read a specific frame from the input video
    ///
    /// Expected to return `Err` if the frame is not found.
    fn read_specific_frame<T: Pixel>(&mut self, frame_number: usize) -> Result<FrameInfo<T>, ()>;
    /// Get the bit depth of the video.
    fn get_bit_depth(&self) -> usize;
}
