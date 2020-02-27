//! Peak Signal-to-Noise Ratio metric.
//!
//! PSNR is most easily defined via the mean squared error between two images.
//!
//! See https://en.wikipedia.org/wiki/Peak_signal-to-noise_ratio for more details.

#[cfg(feature = "decode")]
use crate::video::decode::Decoder;
use crate::video::pixel::CastFromPrimitive;
use crate::video::pixel::Pixel;
use crate::video::{FrameInfo, PlanarMetrics, VideoMetric};
use std::error::Error;
use v_frame::plane::Plane;

/// Calculates the PSNR for two videos. Higher is better.
///
/// PSNR is capped at 100 in order to avoid skewed statistics
/// from e.g. all black frames, which would
/// otherwise show a PSNR of infinity.
#[cfg(feature = "decode")]
#[inline]
pub fn calculate_video_psnr<D: Decoder>(
    decoder1: &mut D,
    decoder2: &mut D,
    frame_limit: Option<usize>,
) -> Result<PlanarMetrics, Box<dyn Error>> {
    let metrics = Psnr.process_video(decoder1, decoder2, frame_limit)?;
    Ok(metrics.psnr)
}

/// Calculates the APSNR for two videos. Higher is better.
///
/// APSNR is capped at 100 in order to avoid skewed statistics
/// from e.g. all black frames, which would
/// otherwise show a APSNR of infinity.
#[cfg(feature = "decode")]
#[inline]
pub fn calculate_video_apsnr<D: Decoder>(
    decoder1: &mut D,
    decoder2: &mut D,
    frame_limit: Option<usize>,
) -> Result<PlanarMetrics, Box<dyn Error>> {
    let metrics = Psnr.process_video(decoder1, decoder2, frame_limit)?;
    Ok(metrics.apsnr)
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
    let metrics = Psnr.process_frame(frame1, frame2)?;
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
        &mut self,
        frame1: &FrameInfo<T>,
        frame2: &FrameInfo<T>,
    ) -> Result<Self::FrameResult, Box<dyn Error>> {
        frame1.can_compare(&frame2)?;

        let bit_depth = frame1.bit_depth;
        let y = calculate_plane_psnr_metrics(&frame1.planes[0], &frame2.planes[0], bit_depth);
        let u = calculate_plane_psnr_metrics(&frame1.planes[1], &frame2.planes[1], bit_depth);
        let v = calculate_plane_psnr_metrics(&frame1.planes[2], &frame2.planes[2], bit_depth);
        Ok([y, u, v])
    }

    #[cfg(feature = "decode")]
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
    if metrics.sq_err <= std::f64::EPSILON {
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
        .map(|(a, b)| (i32::cast_from(*a) - i32::cast_from(*b)).abs() as u64)
        .map(|err| err * err)
        .sum::<u64>() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_metric_eq;
    use std::fs::File;
    use y4m::Decoder;

    #[test]
    fn psnr_yuv420p8() {
        let mut file1 = File::open("./testfiles/yuv420p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv420p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_psnr::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(32.5281, result.y);
        assert_metric_eq(36.4083, result.u);
        assert_metric_eq(39.8238, result.v);
        assert_metric_eq(33.6861, result.avg);
    }

    #[test]
    fn psnr_yuv422p8() {
        let mut file1 = File::open("./testfiles/yuv422p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv422p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_psnr::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(38.6740, result.y);
        assert_metric_eq(47.5219, result.u);
        assert_metric_eq(48.8615, result.v);
        assert_metric_eq(41.2190, result.avg);
    }

    #[test]
    fn psnr_yuv444p8() {
        let mut file1 = File::open("./testfiles/yuv444p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv444p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_psnr::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(32.4235, result.y);
        assert_metric_eq(40.1212, result.u);
        assert_metric_eq(43.1900, result.v);
        assert_metric_eq(36.2126, result.avg);
    }

    #[test]
    fn psnr_yuv420p10() {
        let mut file1 = File::open("./testfiles/yuv420p10_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv420p10_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_psnr::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(32.5421, result.y);
        assert_metric_eq(36.4922, result.u);
        assert_metric_eq(39.8558, result.v);
        assert_metric_eq(33.7071, result.avg);
    }

    #[test]
    fn apsnr_yuv420p8() {
        let mut file1 = File::open("./testfiles/yuv420p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv420p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_apsnr::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(32.5450, result.y);
        assert_metric_eq(36.4087, result.u);
        assert_metric_eq(39.8244, result.v);
        assert_metric_eq(33.6995, result.avg);
    }

    #[test]
    fn apsnr_yuv422p8() {
        let mut file1 = File::open("./testfiles/yuv422p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv422p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_apsnr::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(38.6741, result.y);
        assert_metric_eq(47.5219, result.u);
        assert_metric_eq(48.8616, result.v);
        assert_metric_eq(41.2191, result.avg);
    }

    #[test]
    fn apsnr_yuv444p8() {
        let mut file1 = File::open("./testfiles/yuv444p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv444p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_apsnr::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(32.4412, result.y);
        assert_metric_eq(40.1264, result.u);
        assert_metric_eq(43.1943, result.v);
        assert_metric_eq(36.2271, result.avg);
    }

    #[test]
    fn apsnr_yuv420p10() {
        let mut file1 = File::open("./testfiles/yuv420p10_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv420p10_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_apsnr::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(32.5586, result.y);
        assert_metric_eq(36.4923, result.u);
        assert_metric_eq(39.8563, result.v);
        assert_metric_eq(33.7200, result.avg);
    }
}
