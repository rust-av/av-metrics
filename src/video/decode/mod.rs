use crate::video::pixel::Pixel;
use crate::video::FrameInfo;

#[cfg(feature = "y4m-decode")]
mod y4m;

#[cfg(feature = "y4m-decode")]
pub use self::y4m::*;

pub trait Decoder<T: Pixel> {
    fn read_video_frame(&mut self) -> Result<FrameInfo<T>, ()>;
}
