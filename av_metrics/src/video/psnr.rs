//! Peak Signal-to-Noise Ratio metric.
//!
//! PSNR is most easily defined via the mean squared error between two images.
//!
//! See https://en.wikipedia.org/wiki/Peak_signal-to-noise_ratio for more details.

use crate::video::decode::Decoder;
use crate::video::pixel::CastFromPrimitive;
use crate::video::pixel::Pixel;
use crate::video::{PlanarMetrics, VideoMetric};
use crate::MetricsError;
use std::error::Error;
use std::mem::size_of;
use v_frame::frame::Frame;
use v_frame::plane::Plane;
use v_frame::prelude::ChromaSampling;

use super::FrameCompare;

/// Calculates the PSNR for two videos. Higher is better.
///
/// PSNR is capped at 100 in order to avoid skewed statistics
/// from e.g. all black frames, which would
/// otherwise show a PSNR of infinity.
#[inline]
pub fn calculate_video_psnr<D: Decoder, F: Fn(usize) + Send>(
    decoder1: &mut D,
    decoder2: &mut D,
    frame_limit: Option<usize>,
    progress_callback: F,
) -> Result<PlanarMetrics, Box<dyn Error>> {
    let metrics = Psnr.process_video(decoder1, decoder2, frame_limit, progress_callback)?;
    Ok(metrics.psnr)
}

/// Calculates the APSNR for two videos. Higher is better.
///
/// APSNR is capped at 100 in order to avoid skewed statistics
/// from e.g. all black frames, which would
/// otherwise show a APSNR of infinity.
#[inline]
pub fn calculate_video_apsnr<D: Decoder, F: Fn(usize) + Send>(
    decoder1: &mut D,
    decoder2: &mut D,
    frame_limit: Option<usize>,
    progress_callback: F,
) -> Result<PlanarMetrics, Box<dyn Error>> {
    let metrics = Psnr.process_video(decoder1, decoder2, frame_limit, progress_callback)?;
    Ok(metrics.apsnr)
}

/// Calculates the PSNR for two video frames. Higher is better.
///
/// PSNR is capped at 100 in order to avoid skewed statistics
/// from e.g. all black frames, which would
/// otherwise show a PSNR of infinity.
#[inline]
pub fn calculate_frame_psnr<T: Pixel>(
    frame1: &Frame<T>,
    frame2: &Frame<T>,
    bit_depth: usize,
    chroma_sampling: ChromaSampling,
) -> Result<PlanarMetrics, Box<dyn Error>> {
    let metrics = Psnr.process_frame(frame1, frame2, bit_depth, chroma_sampling)?;
    Ok(PlanarMetrics {
        y: calculate_psnr(metrics[0]),
        u: calculate_psnr(metrics[1]),
        v: calculate_psnr(metrics[2]),
        avg: calculate_summed_psnr(&metrics),
    })
}

#[derive(Debug, Clone, Copy)]
struct PsnrResults {
    psnr: PlanarMetrics,
    apsnr: PlanarMetrics,
}

struct Psnr;

impl VideoMetric for Psnr {
    type FrameResult = [PsnrMetrics; 3];
    type VideoResult = PsnrResults;

    fn process_frame<T: Pixel>(
        &self,
        frame1: &Frame<T>,
        frame2: &Frame<T>,
        bit_depth: usize,
        _chroma_sampling: ChromaSampling,
    ) -> Result<Self::FrameResult, Box<dyn Error>> {
        if (size_of::<T>() == 1 && bit_depth > 8) || (size_of::<T>() == 2 && bit_depth <= 8) {
            return Err(Box::new(MetricsError::InputMismatch {
                reason: "Bit depths does not match pixel width",
            }));
        }

        frame1.can_compare(frame2)?;

        let mut y = Default::default();
        let mut u = Default::default();
        let mut v = Default::default();

        rayon::scope(|s| {
            s.spawn(|_| {
                y = calculate_plane_psnr_metrics(&frame1.planes[0], &frame2.planes[0], bit_depth)
            });
            s.spawn(|_| {
                u = calculate_plane_psnr_metrics(&frame1.planes[1], &frame2.planes[1], bit_depth)
            });
            s.spawn(|_| {
                v = calculate_plane_psnr_metrics(&frame1.planes[2], &frame2.planes[2], bit_depth)
            });
        });

        Ok([y, u, v])
    }

    fn aggregate_frame_results(
        &self,
        metrics: &[Self::FrameResult],
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        let psnr = PlanarMetrics {
            y: calculate_summed_psnr(&metrics.iter().map(|m| m[0]).collect::<Vec<_>>()),
            u: calculate_summed_psnr(&metrics.iter().map(|m| m[1]).collect::<Vec<_>>()),
            v: calculate_summed_psnr(&metrics.iter().map(|m| m[2]).collect::<Vec<_>>()),
            avg: calculate_summed_psnr(&metrics.iter().flatten().copied().collect::<Vec<_>>()),
        };
        let apsnr = PlanarMetrics {
            y: metrics.iter().map(|m| calculate_psnr(m[0])).sum::<f64>() / metrics.len() as f64,
            u: metrics.iter().map(|m| calculate_psnr(m[1])).sum::<f64>() / metrics.len() as f64,
            v: metrics.iter().map(|m| calculate_psnr(m[2])).sum::<f64>() / metrics.len() as f64,
            avg: metrics
                .iter()
                .map(|m| calculate_summed_psnr(m))
                .sum::<f64>()
                / metrics.len() as f64,
        };
        Ok(PsnrResults { psnr, apsnr })
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct PsnrMetrics {
    sq_err: f64,
    n_pixels: usize,
    sample_max: usize,
}

fn calculate_summed_psnr(metrics: &[PsnrMetrics]) -> f64 {
    calculate_psnr(
        metrics
            .iter()
            .fold(PsnrMetrics::default(), |acc, plane| PsnrMetrics {
                sq_err: acc.sq_err + plane.sq_err,
                sample_max: plane.sample_max,
                n_pixels: acc.n_pixels + plane.n_pixels,
            }),
    )
}

/// Calculate the PSNR metrics for a `Plane` by comparing the original (uncompressed) to
/// the compressed version.
fn calculate_plane_psnr_metrics<T: Pixel>(
    plane1: &Plane<T>,
    plane2: &Plane<T>,
    bit_depth: usize,
) -> PsnrMetrics {
    let sq_err = calculate_plane_total_squared_error(plane1, plane2);
    let max = (1 << bit_depth) - 1;
    PsnrMetrics {
        sq_err,
        n_pixels: plane1.cfg.width * plane1.cfg.height,
        sample_max: max,
    }
}

fn calculate_psnr(metrics: PsnrMetrics) -> f64 {
    if metrics.sq_err <= f64::EPSILON {
        return 100.0;
    }
    10.0 * ((metrics.sample_max.pow(2) as f64).log10() + (metrics.n_pixels as f64).log10()
        - metrics.sq_err.log10())
}

/// Calculate the squared error for a `Plane` by comparing the original (uncompressed)
/// to the compressed version.
fn calculate_plane_total_squared_error<T: Pixel>(plane1: &Plane<T>, plane2: &Plane<T>) -> f64 {
    plane1
        .data
        .iter()
        .zip(plane2.data.iter())
        .map(|(a, b)| (i32::cast_from(*a) - i32::cast_from(*b)).unsigned_abs() as u64)
        .map(|err| err * err)
        .sum::<u64>() as f64
}
