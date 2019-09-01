mod ciede;
#[cfg(feature = "decode")]
pub mod decode;
pub mod pixel;
mod psnr;
mod psnr_hvs;
mod ssim;

use crate::video::pixel::Pixel;
use crate::MetricsError;
pub use ciede::*;
pub use psnr::*;
pub use psnr_hvs::*;
pub use ssim::*;

#[derive(Clone, Debug)]
pub struct FrameInfo<T: Pixel> {
    /// A container holding three planes worth of video data.
    /// The indices in the array correspond to the following planes:
    ///
    /// 0 - Y/Luma plane
    /// 1 - U/Cb plane
    /// 2 - V/Cr plane
    pub planes: [PlaneData<T>; 3],
    pub bit_depth: usize,
    pub chroma_sampling: ChromaSampling,
}

impl<T: Pixel> FrameInfo<T> {
    pub(crate) fn can_compare(&self, other: &Self) -> Result<(), MetricsError> {
        if self.bit_depth != other.bit_depth {
            return Err(MetricsError::InputMismatch {
                reason: "Bit depths do not match",
            });
        }
        if self.bit_depth > 16 {
            return Err(MetricsError::UnsupportedInput {
                reason: "Bit depths above 16 are not supported",
            });
        }
        if self.chroma_sampling != other.chroma_sampling {
            return Err(MetricsError::InputMismatch {
                reason: "Chroma subsampling offsets do not match",
            });
        }
        self.planes[0].can_compare(&other.planes[0])?;
        self.planes[1].can_compare(&other.planes[1])?;
        self.planes[2].can_compare(&other.planes[2])?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct PlaneData<T: Pixel> {
    /// The width, in pixels, of this plane.
    pub width: usize,
    /// The height, in pixels, of this plane.
    pub height: usize,
    /// Each plane's pixels should be contained in a `Vec`, in row-major order.
    /// A `u8` should be used for low-bit-depth video, and `u16` for high-bit-depth.
    pub data: Vec<T>,
}

impl<T: Pixel> PlaneData<T> {
    pub(crate) fn can_compare(&self, other: &Self) -> Result<(), MetricsError> {
        if self.width != other.width || self.height != other.height {
            return Err(MetricsError::InputMismatch {
                reason: "Video resolution does not match",
            });
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ChromaSampling {
    /// Both vertically and horizontally subsampled.
    Cs420,
    /// Horizontally subsampled.
    Cs422,
    /// Not subsampled.
    Cs444,
    /// Monochrome.
    Cs400,
}

impl Default for ChromaSampling {
    fn default() -> Self {
        ChromaSampling::Cs420
    }
}

impl ChromaSampling {
    /// Provides the amount to right shift the luma plane dimensions to get the
    ///  chroma plane dimensions.
    /// Only values 0 or 1 are ever returned.
    /// The plane dimensions must also be rounded up to accommodate odd luma plane
    ///  sizes.
    /// Cs400 returns None, as there are no chroma planes.
    pub(crate) fn get_decimation(self) -> Option<(usize, usize)> {
        use self::ChromaSampling::*;
        match self {
            Cs420 => Some((1, 1)),
            Cs422 => Some((1, 0)),
            Cs444 => Some((0, 0)),
            Cs400 => None,
        }
    }

    /// Calculates the size of a chroma plane for this sampling type, given the luma plane dimensions.
    pub(crate) fn get_chroma_dimensions(
        self,
        luma_width: usize,
        luma_height: usize,
    ) -> (usize, usize) {
        if let Some((ss_x, ss_y)) = self.get_decimation() {
            ((luma_width + ss_x) >> ss_x, (luma_height + ss_y) >> ss_y)
        } else {
            (0, 0)
        }
    }

    /// The relative impact of chroma planes compared to luma
    pub(crate) fn get_chroma_weight(self) -> f64 {
        match self {
            ChromaSampling::Cs420 => 0.25,
            ChromaSampling::Cs422 => 0.5,
            ChromaSampling::Cs444 => 1.0,
            ChromaSampling::Cs400 => 0.0,
        }
    }
}

/// Sample position for subsampled chroma
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ChromaSamplePosition {
    /// The source video transfer function must be signaled
    /// outside the AV1 bitstream.
    Unknown,
    /// Horizontally co-located with (0, 0) luma sample, vertically positioned
    /// in the middle between two luma samples.
    Vertical,
    /// Co-located with (0, 0) luma sample.
    Colocated,
    /// Bilaterally located chroma plane in the diagonal space between luma samples.
    Bilateral,
    /// Interlaced content with interpolated chroma samples
    Interpolated,
}

impl Default for ChromaSamplePosition {
    fn default() -> Self {
        ChromaSamplePosition::Unknown
    }
}

/// Certain metrics return a value per plane. This struct contains the output
/// for those metrics per plane, as well as a weighted average of the planes.
#[derive(Debug, Clone, Copy)]
pub struct PlanarMetrics {
    /// Metric value for the Y plane.
    pub y: f64,
    /// Metric value for the U/Cb plane.
    pub u: f64,
    /// Metric value for the V/Cb plane.
    pub v: f64,
    /// Weighted average of the three planes for this frame
    pub avg: f64,
}
