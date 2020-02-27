use crate::video::pixel::Pixel;
use crate::video::FrameInfo;
use crate::video::{ChromaSamplePosition, ChromaSampling};

#[cfg(feature = "y4m-decode")]
mod y4m;

#[cfg(feature = "y4m-decode")]
pub use self::y4m::*;

/// A trait for allowing metrics to decode generic video formats.
///
/// Currently, y4m decoding support using the `y4m` crate is built-in
/// to this crate. This trait is extensible so users may implement
/// their own decoders.
pub trait Decoder {
    /// Read the next frame from the input video.
    ///
    /// Expected to return `Err` if the end of the video is reached.
    fn read_video_frame<T: Pixel>(&mut self, cfg: &VideoDetails) -> Result<FrameInfo<T>, ()>;
    /// Read a specific frame from the input video
    ///
    /// Expected to return `Err` if the frame is not found.
    fn read_specific_frame<T: Pixel>(&mut self, frame_number: usize) -> Result<FrameInfo<T>, ()>;
    /// Get the bit depth of the video.
    fn get_bit_depth(&self) -> usize;
    /// Get the Video Details
    fn get_video_details(&self) -> VideoDetails;
}

/// A Structure containing Video Details as per Plane's Config
#[derive(Debug, Clone, Copy)]
pub struct VideoDetails {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
    /// Bit-depth of the Video
    pub bit_depth: usize,
    /// ChromaSampling of the Video.
    pub chroma_sampling: ChromaSampling,
    /// Chroma Sampling Position of the Video.
    pub chroma_sample_position: ChromaSamplePosition,
    /// Add Time base of the Video.
    pub time_base: Rational,
    /// Padding Constant
    pub luma_padding: usize,
}

impl Default for VideoDetails {
    fn default() -> Self {
        VideoDetails {
            width: 640,
            height: 480,
            bit_depth: 8,
            chroma_sampling: ChromaSampling::Cs420,
            chroma_sample_position: ChromaSamplePosition::Unknown,
            time_base: Rational { num: 30, den: 1 },
            luma_padding: 0,
        }
    }
}

/// A rational number.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Rational {
    /// Numerator.
    pub num: u64,
    /// Denominator.
    pub den: u64,
}

impl Rational {
    /// Creates a rational number from the given numerator and denominator.
    pub const fn new(num: u64, den: u64) -> Self {
        Rational { num, den }
    }
    /// Returns a rational number that is the reciprocal of the given one.
    pub const fn from_reciprocal(reciprocal: Self) -> Self {
        Rational {
            num: reciprocal.den,
            den: reciprocal.num,
        }
    }
    /// Returns the rational number as a floating-point number.
    pub fn as_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }
}
