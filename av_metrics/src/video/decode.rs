//! Contains a trait and utilities for implementing decoders.
//! Prebuilt decoders are included in the `av-metrics-decoders` crate.

use crate::video::pixel::Pixel;
use crate::video::{ChromaSamplePosition, ChromaSampling};
use std::cmp;
use v_frame::frame::Frame;
use v_frame::pixel::CastFromPrimitive;
use v_frame::plane::Plane;

/// A trait for allowing metrics to decode generic video formats.
///
/// Currently, y4m decoding support using the `y4m` crate is built-in
/// to this crate. This trait is extensible so users may implement
/// their own decoders.
pub trait Decoder: Send {
    /// Read the next frame from the input video.
    ///
    /// Expected to return `Err` if the end of the video is reached.
    fn read_video_frame<T: Pixel>(&mut self) -> Option<Frame<T>>;
    /// Read a specific frame from the input video
    ///
    /// Expected to return `Err` if the frame is not found.
    fn read_specific_frame<T: Pixel>(&mut self, frame_number: usize) -> Option<Frame<T>> {
        let mut frame_no = 0;
        while frame_no <= frame_number {
            let frame = self.read_video_frame();
            if frame_no == frame_number && frame.is_some() {
                return frame;
            }
            frame_no += 1;
        }
        None
    }
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

/// The algorithms (as ported from daala-tools) expect a colocated or bilaterally located chroma
/// sample position. This means that a vertical chroma sample position must be realigned
/// in order to produce a correct result.
pub fn convert_chroma_data<T: Pixel>(
    plane_data: &mut Plane<T>,
    chroma_pos: ChromaSamplePosition,
    bit_depth: usize,
    source: &[u8],
    source_stride: usize,
    source_bytewidth: usize,
) {
    if chroma_pos != ChromaSamplePosition::Vertical {
        // TODO: Also convert Interpolated chromas
        plane_data.copy_from_raw_u8(source, source_stride, source_bytewidth);
        return;
    }

    let get_pixel = if source_bytewidth == 1 {
        fn convert_u8(line: &[u8], index: usize) -> i32 {
            i32::cast_from(line[index])
        }
        convert_u8
    } else {
        fn convert_u16(line: &[u8], index: usize) -> i32 {
            let index = index * 2;
            i32::cast_from(u16::cast_from(line[index + 1]) << 8 | u16::cast_from(line[index]))
        }
        convert_u16
    };

    let output_data = &mut plane_data.data;
    let width = plane_data.cfg.width;
    let height = plane_data.cfg.height;
    for y in 0..height {
        // Filter: [4 -17 114 35 -9 1]/128, derived from a 6-tap Lanczos window.
        let in_row = &source[(y * source_stride)..];
        let out_row = &mut output_data[(y * width)..];
        let breakpoint = cmp::min(width, 2);
        for x in 0..breakpoint {
            out_row[x] = T::cast_from(clamp(
                (4 * get_pixel(in_row, 0) - 17 * get_pixel(in_row, x.saturating_sub(1))
                    + 114 * get_pixel(in_row, x)
                    + 35 * get_pixel(in_row, cmp::min(x + 1, width - 1))
                    - 9 * get_pixel(in_row, cmp::min(x + 2, width - 1))
                    + get_pixel(in_row, cmp::min(x + 3, width - 1))
                    + 64)
                    >> 7,
                0,
                (1 << bit_depth) - 1,
            ));
        }
        let breakpoint2 = width - 3;
        for x in breakpoint..breakpoint2 {
            out_row[x] = T::cast_from(clamp(
                (4 * get_pixel(in_row, x - 2) - 17 * get_pixel(in_row, x - 1)
                    + 114 * get_pixel(in_row, x)
                    + 35 * get_pixel(in_row, x + 1)
                    - 9 * get_pixel(in_row, x + 2)
                    + get_pixel(in_row, x + 3)
                    + 64)
                    >> 7,
                0,
                (1 << bit_depth) - 1,
            ));
        }
        for x in breakpoint2..width {
            out_row[x] = T::cast_from(clamp(
                (4 * get_pixel(in_row, x - 2) - 17 * get_pixel(in_row, x - 1)
                    + 114 * get_pixel(in_row, x)
                    + 35 * get_pixel(in_row, cmp::min(x + 1, width - 1))
                    - 9 * get_pixel(in_row, cmp::min(x + 2, width - 1))
                    + get_pixel(in_row, width - 1)
                    + 64)
                    >> 7,
                0,
                (1 << bit_depth) - 1,
            ));
        }
    }
}

#[inline]
fn clamp<T: PartialOrd>(input: T, min: T, max: T) -> T {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}
