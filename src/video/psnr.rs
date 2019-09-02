//! Peak Signal-to-Noise Ratio metric.
//!
//! PSNR is most easily defined via the mean squared error between two images.
//!
//! See https://en.wikipedia.org/wiki/Peak_signal-to-noise_ratio for more details.

#[cfg(feature = "decode")]
use crate::video::decode::Decoder;
use crate::video::pixel::CastFromPrimitive;
use crate::video::pixel::Pixel;
use crate::video::{FrameInfo, PlanarMetrics, PlaneData};
#[cfg(feature = "decode")]
use crate::MetricsError;
use std::error::Error;

/// Contains different methods of calculating PSNR over a pair of videos.
/// Each method uses the same per-frame metrics, but combines them differently.
#[derive(Debug, Clone, Copy)]
pub struct PsnrResults {
    /// The standard PSNR calculation
    pub psnr: PlanarMetrics,
    /// Frame-averaged PSNR calculation, known as APSNR
    pub apsnr: PlanarMetrics,
}

/// Calculates the PSNR for two videos. Higher is better.
///
/// PSNR is capped at 100 in order to avoid skewed statistics
/// from e.g. all black frames, which would
/// otherwise show a PSNR of infinity.
#[cfg(feature = "decode")]
#[inline]
pub fn calculate_video_psnr<D: Decoder<T>, T: Pixel>(
    decoder1: &mut D,
    decoder2: &mut D,
    frame_limit: Option<usize>,
) -> Result<PsnrResults, Box<dyn Error>> {
    let mut metrics = Vec::with_capacity(frame_limit.unwrap_or(0));
    let mut frame_no = 0;
    while frame_limit.map(|limit| limit > frame_no).unwrap_or(true) {
        let frame1 = decoder1.read_video_frame();
        let frame2 = decoder2.read_video_frame();
        if let Ok(frame1) = frame1 {
            if let Ok(frame2) = frame2 {
                metrics.push(calculate_frame_psnr_inner(&frame1, &frame2)?);
                frame_no += 1;
                continue;
            }
        }
        // At end of video
        break;
    }
    if frame_no == 0 {
        return Err(MetricsError::UnsupportedInput {
            reason: "No readable frames found in one or more input files",
        }
        .into());
    }

    let psnr = PlanarMetrics {
        y: calculate_summed_psnr(&metrics.iter().map(|m| m[0]).collect::<Vec<_>>()),
        u: calculate_summed_psnr(&metrics.iter().map(|m| m[1]).collect::<Vec<_>>()),
        v: calculate_summed_psnr(&metrics.iter().map(|m| m[2]).collect::<Vec<_>>()),
        avg: calculate_summed_psnr(&metrics.iter().flatten().copied().collect::<Vec<_>>()),
    };
    let apsnr = PlanarMetrics {
        y: metrics.iter().map(|m| calculate_psnr(m[0])).sum::<f64>() / frame_no as f64,
        u: metrics.iter().map(|m| calculate_psnr(m[1])).sum::<f64>() / frame_no as f64,
        v: metrics.iter().map(|m| calculate_psnr(m[2])).sum::<f64>() / frame_no as f64,
        avg: metrics
            .iter()
            .map(|m| calculate_summed_psnr(m))
            .sum::<f64>()
            / frame_no as f64,
    };
    Ok(PsnrResults { psnr, apsnr })
}

/// Calculates the PSNR for two video frames. Higher is better.
///
/// PSNR is capped at 100 in order to avoid skewed statistics
/// from e.g. all black frames, which would
/// otherwise show a PSNR of infinity.
#[inline]
pub fn calculate_frame_psnr<T: Pixel>(
    frame1: &FrameInfo<T>,
    frame2: &FrameInfo<T>,
) -> Result<PlanarMetrics, Box<dyn Error>> {
    let metrics = calculate_frame_psnr_inner(frame1, frame2)?;
    Ok(PlanarMetrics {
        y: calculate_psnr(metrics[0]),
        u: calculate_psnr(metrics[1]),
        v: calculate_psnr(metrics[2]),
        avg: calculate_summed_psnr(&metrics),
    })
}

#[derive(Debug, Clone, Copy, Default)]
struct PsnrMetrics {
    sq_err: f64,
    n_pixels: usize,
    sample_max: usize,
}

fn calculate_frame_psnr_inner<T: Pixel>(
    frame1: &FrameInfo<T>,
    frame2: &FrameInfo<T>,
) -> Result<[PsnrMetrics; 3], Box<dyn Error>> {
    frame1.can_compare(&frame2)?;

    let bit_depth = frame1.bit_depth;
    let y = calculate_plane_psnr_metrics(&frame1.planes[0], &frame2.planes[0], bit_depth);
    let u = calculate_plane_psnr_metrics(&frame1.planes[1], &frame2.planes[1], bit_depth);
    let v = calculate_plane_psnr_metrics(&frame1.planes[2], &frame2.planes[2], bit_depth);
    Ok([y, u, v])
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
    plane1: &PlaneData<T>,
    plane2: &PlaneData<T>,
    bit_depth: usize,
) -> PsnrMetrics {
    let sq_err = calculate_plane_total_squared_error(plane1, plane2);
    let max = (1 << bit_depth) - 1;
    PsnrMetrics {
        sq_err,
        n_pixels: plane1.width * plane1.height,
        sample_max: max,
    }
}

fn calculate_psnr(metrics: PsnrMetrics) -> f64 {
    if metrics.sq_err <= std::f64::EPSILON {
        return 100.0;
    }
    10.0 * ((metrics.sample_max.pow(2) as f64).log10() + (metrics.n_pixels as f64).log10()
        - metrics.sq_err.log10())
}

/// Calculate the squared error for a `Plane` by comparing the original (uncompressed)
/// to the compressed version.
fn calculate_plane_total_squared_error<T: Pixel>(
    plane1: &PlaneData<T>,
    plane2: &PlaneData<T>,
) -> f64 {
    plane1
        .data
        .iter()
        .zip(plane2.data.iter())
        .map(|(a, b)| (i32::cast_from(*a) - i32::cast_from(*b)).abs() as u64)
        .map(|err| err * err)
        .sum::<u64>() as f64
}
