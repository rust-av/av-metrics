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

trait FrameCompare {
    fn can_compare(&self, other: &Self) -> Result<(), MetricsError>;
}

impl<T: Pixel> FrameCompare for Frame<T> {
    fn can_compare(&self, other: &Self) -> Result<(), MetricsError> {
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
#[derive(Debug, Default, Clone, Copy, PartialEq)]
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
    fn process_video<D: Decoder, F: Fn(usize) + Send>(
        &mut self,
        decoder1: &mut D,
        decoder2: &mut D,
        frame_limit: Option<usize>,
        progress_callback: F,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        if decoder1.get_bit_depth() != decoder2.get_bit_depth() {
            return Err(Box::new(MetricsError::InputMismatch {
                reason: "Bit depths do not match",
            }));
        }
        if decoder1.get_video_details().chroma_sampling
            != decoder2.get_video_details().chroma_sampling
        {
            return Err(Box::new(MetricsError::InputMismatch {
                reason: "Chroma samplings do not match",
            }));
        }

        if decoder1.get_bit_depth() > 8 {
            self.process_video_mt::<D, u16, F>(decoder1, decoder2, frame_limit, progress_callback)
        } else {
            self.process_video_mt::<D, u8, F>(decoder1, decoder2, frame_limit, progress_callback)
        }
    }

    fn process_frame<T: Pixel>(
        &self,
        frame1: &Frame<T>,
        frame2: &Frame<T>,
        bit_depth: usize,
        chroma_sampling: ChromaSampling,
    ) -> Result<Self::FrameResult, Box<dyn Error>>;

    fn aggregate_frame_results(
        &self,
        metrics: &[Self::FrameResult],
    ) -> Result<Self::VideoResult, Box<dyn Error>>;

    fn process_video_mt<D: Decoder, P: Pixel, F: Fn(usize) + Send>(
        &mut self,
        decoder1: &mut D,
        decoder2: &mut D,
        frame_limit: Option<usize>,
        progress_callback: F,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        let num_threads = (rayon::current_num_threads() - 1).max(1);

        let mut out = Vec::new();

        let (send, recv) = crossbeam::channel::bounded(num_threads);
        let vid_info = decoder1.get_video_details();

        match crossbeam::scope(|s| {
            let send_result = s.spawn(move |_| {
                let mut decoded = 0;
                while frame_limit.map(|limit| limit > decoded).unwrap_or(true) {
                    decoded += 1;
                    let frame1 = decoder1.read_video_frame::<P>();
                    let frame2 = decoder2.read_video_frame::<P>();
                    if let (Some(frame1), Some(frame2)) = (frame1, frame2) {
                        progress_callback(decoded);
                        if let Err(e) = send.send((frame1, frame2)) {
                            let (frame1, frame2) = e.into_inner();
                            return Err(format!(
                                "Error sending\n\nframe1: {:?}\n\nframe2: {:?}",
                                frame1, frame2
                            ));
                        }
                    } else {
                        break;
                    }
                }
                // Mark the end of the decoding process
                progress_callback(usize::MAX);
                Ok(())
            });

            use rayon::prelude::*;
            let mut metrics = Vec::with_capacity(frame_limit.unwrap_or(0));
            let mut process_error = Ok(());
            loop {
                let working_set: Vec<_> = (0..num_threads)
                    .into_par_iter()
                    .filter_map(|_w| {
                        recv.recv()
                            .map(|(f1, f2)| {
                                self.process_frame(
                                    &f1,
                                    &f2,
                                    vid_info.bit_depth,
                                    vid_info.chroma_sampling,
                                )
                                .map_err(|e| {
                                    format!(
                                        "\n\n{} on\n\nframe1: {:?}\n\nand\n\nframe2: {:?}",
                                        e, f1, f2
                                    )
                                })
                            })
                            .ok()
                    })
                    .collect();
                let work_set: Vec<_> = working_set
                    .into_iter()
                    .filter_map(|v| v.map_err(|e| process_error = Err(e)).ok())
                    .collect();
                if work_set.is_empty() || process_error.is_err() {
                    break;
                } else {
                    metrics.extend(work_set);
                }
            }

            out = metrics;

            (
                send_result
                    .join()
                    .unwrap_or_else(|_| Err("Failed joining the sender thread".to_owned())),
                process_error,
            )
        }) {
            Ok((send_error, process_error)) => {
                if let Err(error) = send_error {
                    return Err(MetricsError::SendError { reason: error }.into());
                }

                if let Err(error) = process_error {
                    return Err(MetricsError::ProcessError { reason: error }.into());
                }

                if out.is_empty() {
                    return Err(MetricsError::UnsupportedInput {
                        reason: "No readable frames found in one or more input files",
                    }
                    .into());
                }

                self.aggregate_frame_results(&out)
            }
            Err(e) => Err(MetricsError::VideoError {
                reason: format!("\n\nError {:?} processing the two videos", e),
            }
            .into()),
        }
    }
}
