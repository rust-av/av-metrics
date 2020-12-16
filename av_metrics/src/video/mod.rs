//! Contains metrics related to video/image quality.

pub mod ciede;
pub mod decode;
mod pixel;
pub mod psnr;
pub mod psnr_hvs;
pub mod ssim;

use crate::MetricsError;
use decode::*;
use std::error::Error;

pub use pixel::*;
pub use v_frame::frame::Frame;
pub use v_frame::plane::Plane;

/// A container holding the data for one video frame. This includes all planes
/// of the video. Currently, only YUV/YCbCr format is supported. Bit depths up to 16-bit
/// are supported.
#[derive(Clone, Debug)]
pub struct FrameInfo<T: Pixel> {
    /// A container holding three planes worth of video data.
    /// The indices in the array correspond to the following planes:
    ///
    /// - 0 - Y/Luma plane
    /// - 1 - U/Cb plane
    /// - 2 - V/Cr plane
    pub planes: [Plane<T>; 3],
    /// The number of bits per pixel.
    pub bit_depth: usize,
    /// The chroma sampling format of the video. Most videos are in 4:2:0 format.
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

pub(crate) trait PlaneCompare {
    fn can_compare(&self, other: &Self) -> Result<(), MetricsError>;
}

impl<T: Pixel> PlaneCompare for Plane<T> {
    fn can_compare(&self, other: &Self) -> Result<(), MetricsError> {
        if self.cfg != other.cfg {
            return Err(MetricsError::InputMismatch {
                reason: "Video resolution does not match",
            });
        }
        Ok(())
    }
}

pub use v_frame::pixel::ChromaSampling;

pub(crate) trait ChromaWeight {
    fn get_chroma_weight(self) -> f64;
}

impl ChromaWeight for ChromaSampling {
    /// The relative impact of chroma planes compared to luma
    fn get_chroma_weight(self) -> f64 {
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
    /// The source video transfer function is not signaled. This crate will assume
    /// no transformation needs to be done on this data, but there is a risk of metric
    /// calculations being inaccurate.
    Unknown,
    /// Horizontally co-located with (0, 0) luma sample, vertically positioned
    /// in the middle between two luma samples.
    Vertical,
    /// Co-located with (0, 0) luma sample.
    Colocated,
    /// Bilaterally located chroma plane in the diagonal space between luma samples.
    Bilateral,
    /// Interlaced content with interpolated chroma samples.
    Interpolated,
}

impl Default for ChromaSamplePosition {
    fn default() -> Self {
        ChromaSamplePosition::Unknown
    }
}

/// Certain metrics return a value per plane. This struct contains the output
/// for those metrics per plane, as well as a weighted average of the planes.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PlanarMetrics {
    /// Metric value for the Y plane.
    pub y: f64,
    /// Metric value for the U/Cb plane.
    pub u: f64,
    /// Metric value for the V/Cb plane.
    pub v: f64,
    /// Weighted average of the three planes.
    pub avg: f64,
}

trait VideoMetric: Send + Sync {
    type FrameResult: Send + Sync;
    type VideoResult: Send + Sync;

    /// Generic method for internal use that processes multiple frames from a video
    /// into an aggregate metric.
    ///
    /// `frame_fn` is the function to calculate metrics on one frame of the video.
    /// `acc_fn` is the accumulator function to calculate the aggregate metric.
    fn process_video<D: Decoder>(
        &mut self,
        decoder1: &mut D,
        decoder2: &mut D,
        frame_limit: Option<usize>,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        if decoder1.get_bit_depth() != decoder2.get_bit_depth() {
            return Err(Box::new(MetricsError::InputMismatch {
                reason: "Bit depths do not match",
            }));
        }

        if decoder1.get_bit_depth() > 8 {
            self.process_video_mt::<D, u16>(decoder1, decoder2, frame_limit)
        } else {
            self.process_video_mt::<D, u8>(decoder1, decoder2, frame_limit)
        }
    }

    fn process_frame<T: Pixel>(
        &self,
        frame1: &FrameInfo<T>,
        frame2: &FrameInfo<T>,
    ) -> Result<Self::FrameResult, Box<dyn Error>>;

    fn aggregate_frame_results(
        &self,
        metrics: &[Self::FrameResult],
    ) -> Result<Self::VideoResult, Box<dyn Error>>;

    fn process_video_mt<D: Decoder, P: Pixel>(
        &mut self,
        decoder1: &mut D,
        decoder2: &mut D,
        frame_limit: Option<usize>,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        let num_threads = (rayon::current_num_threads() - 1).max(1);

        let mut out = Vec::new();

        let (send, recv) = crossbeam::channel::bounded(num_threads);

        crossbeam::scope(|s| {
            s.spawn(move |_| {
                let mut decoded = 0;
                while frame_limit.map(|limit| limit > decoded).unwrap_or(true) {
                    decoded += 1;
                    let frame1 = decoder1.read_video_frame::<P>();
                    let frame2 = decoder2.read_video_frame::<P>();
                    if let (Ok(frame1), Ok(frame2)) = (frame1, frame2) {
                        send.send((frame1, frame2)).unwrap();
                    } else {
                        break;
                    }
                }
            });

            use rayon::prelude::*;
            let mut metrics = Vec::with_capacity(frame_limit.unwrap_or(0));
            loop {
                let working_set: Vec<_> = (0..num_threads)
                    .into_par_iter()
                    .filter_map(|_w| {
                        recv.recv()
                            .map(|(f1, f2)| self.process_frame(&f1, &f2).unwrap())
                            .ok()
                    })
                    .collect();
                if working_set.is_empty() {
                    break;
                } else {
                    metrics.extend(working_set);
                }
            }

            out = metrics;
        })
        .unwrap();

        if out.is_empty() {
            return Err(MetricsError::UnsupportedInput {
                reason: "No readable frames found in one or more input files",
            }
            .into());
        }

        self.aggregate_frame_results(&out)
    }
}
