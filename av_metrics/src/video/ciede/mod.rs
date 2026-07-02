#![allow(clippy::cast_ptr_alignment)]

//! The CIEDE2000 color difference formula.
//!
//! CIEDE2000 implementation adapted from
//! [Kyle Siefring's](https://github.com/KyleSiefring/dump_ciede2000).

use crate::video::decode::Decoder;
use crate::video::pixel::Pixel;
use crate::video::ChromaSubsampling;
use crate::video::VideoMetric;
use crate::MetricsError;

use super::FrameCompare;

use std::error::Error;
use std::f64;
use std::mem::size_of;

mod rgbtolab;
use rgbtolab::*;

mod delta_e;
use delta_e::*;

/// Calculate the CIEDE2000 metric between two video clips. Higher is better.
///
/// This will return at the end of the shorter of the two clips,
/// comparing any frames up to that point.
///
/// Optionally, `frame_limit` can be set to only compare the first
/// `frame_limit` frames in each video.
#[inline]
pub fn calculate_video_ciede<D: Decoder, F: Fn(usize) + Send>(
    decoder1: &mut D,
    decoder2: &mut D,
    frame_limit: Option<usize>,
    progress_callback: F,
) -> Result<f64, Box<dyn Error>> {
    Ciede2000::default().process_video(decoder1, decoder2, frame_limit, progress_callback)
}

/// Calculate the CIEDE2000 metric between two video clips. Higher is better.
///
/// This version disables SIMD. It is intended to only be used
/// by tests and benchmarks.
#[inline]
#[doc(hidden)]
pub fn calculate_video_ciede_nosimd<D: Decoder, F: Fn(usize) + Send>(
    decoder1: &mut D,
    decoder2: &mut D,
    frame_limit: Option<usize>,
    progress_callback: F,
) -> Result<f64, Box<dyn Error>> {
    (Ciede2000 { use_simd: false }).process_video(
        decoder1,
        decoder2,
        frame_limit,
        progress_callback,
    )
}

/// Calculate the CIEDE2000 metric between two video frames. Higher is better.
#[inline]
pub fn calculate_frame_ciede<T: Pixel>(
    frame1: &Frame<T>,
    frame2: &Frame<T>,
    bit_depth: usize,
    chroma_sampling: ChromaSubsampling,
) -> Result<f64, Box<dyn Error>> {
    Ciede2000::default().process_frame(frame1, frame2, bit_depth, chroma_sampling)
}

/// Calculate the CIEDE2000 metric between two video frames. Higher is better.
///
/// This version disables SIMD. It is intended to only be used
/// by tests and benchmarks.
#[inline]
#[doc(hidden)]
pub fn calculate_frame_ciede_nosimd<T: Pixel>(
    frame1: &Frame<T>,
    frame2: &Frame<T>,
    bit_depth: usize,
    chroma_sampling: ChromaSubsampling,
) -> Result<f64, Box<dyn Error>> {
    (Ciede2000 { use_simd: false }).process_frame(frame1, frame2, bit_depth, chroma_sampling)
}

struct Ciede2000 {
    use_simd: bool,
}

impl Default for Ciede2000 {
    fn default() -> Self {
        Ciede2000 { use_simd: true }
    }
}

use rayon::prelude::*;
use v_frame::frame::Frame;

impl VideoMetric for Ciede2000 {
    type FrameResult = f64;
    type VideoResult = f64;

    fn process_frame<T: Pixel>(
        &self,
        frame1: &Frame<T>,
        frame2: &Frame<T>,
        bit_depth: usize,
        chroma_sampling: ChromaSubsampling,
    ) -> Result<Self::FrameResult, Box<dyn Error>> {
        if (size_of::<T>() == 1 && bit_depth > 8) || (size_of::<T>() == 2 && bit_depth <= 8) {
            return Err(Box::new(MetricsError::InputMismatch {
                reason: "Bit depths does not match pixel width",
            }));
        }

        frame1.can_compare(frame2)?;

        let (x_ratio, y_ratio) = chroma_sampling.subsample_ratio().expect("not monochrome");
        let y_width = frame1.y_plane.width();
        let y_height = frame1.y_plane.height();
        let c_width = frame1.plane(1).expect("has U plane").width();
        let delta_e_row_fn = get_delta_e_row_fn(bit_depth, x_ratio.get(), self.use_simd);
        // let mut delta_e_vec: Vec<f32> = vec![0.0; y_width * y_height];

        let delta_e_per_line = (0..y_height).into_par_iter().map(|i| {
            let y_start = i * y_width;
            let y_end = y_start + y_width;
            let c_start = (i / usize::from(y_ratio.get())) * c_width;
            let c_end = c_start + c_width;

            let y_range = y_start..y_end;
            let c_range = c_start..c_end;

            let mut delta_e_vec = vec![0.0; y_end - y_start];

            unsafe {
                delta_e_row_fn(
                    FrameRow {
                        y: &frame1.plane(0).expect("frame 1 has plane 0").data()[y_range.clone()],
                        u: &frame1.plane(1).expect("frame 1 has plane 1").data()[c_range.clone()],
                        v: &frame1.plane(2).expect("frame 1 has plane 2").data()[c_range.clone()],
                    },
                    FrameRow {
                        y: &frame2.plane(0).expect("frame 2 has plane 0").data()[y_range],
                        u: &frame2.plane(1).expect("frame 2 has plane 1").data()[c_range.clone()],
                        v: &frame2.plane(2).expect("frame 2 has plane 2").data()[c_range],
                    },
                    &mut delta_e_vec[..],
                );
            }

            delta_e_vec.iter().map(|x| *x as f64).sum::<f64>()
        });

        let score =
            45. - 20. * (delta_e_per_line.sum::<f64>() / ((y_width * y_height) as f64)).log10();
        Ok(score.min(100.))
    }

    fn aggregate_frame_results(
        &self,
        metrics: &[Self::FrameResult],
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        Ok(metrics.iter().copied().sum::<f64>() / metrics.len() as f64)
    }
}

// Arguments for delta e
// "Color Image Quality Assessment Based on CIEDE2000"
// Yang Yang, Jun Ming and Nenghai Yu, 2012
// http://dx.doi.org/10.1155/2012/273723
const K_SUB: KSubArgs = KSubArgs {
    l: 0.65,
    c: 1.0,
    h: 4.0,
};

pub(crate) struct FrameRow<'a, T: Pixel> {
    y: &'a [T],
    u: &'a [T],
    v: &'a [T],
}

type DeltaERowFn<T> = unsafe fn(FrameRow<T>, FrameRow<T>, &mut [f32]);

fn get_delta_e_row_fn<T: Pixel>(bit_depth: usize, x_ratio: u8, _simd: bool) -> DeltaERowFn<T> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") && x_ratio == 2 && _simd {
            return match bit_depth {
                8 => avx2::delta_e_row_avx2::<T, BD8>,
                10 => avx2::delta_e_row_avx2::<T, BD10>,
                12 => avx2::delta_e_row_avx2::<T, BD12>,
                _ => unreachable!(),
            };
        }
    }
    match (bit_depth, x_ratio) {
        (8, 2) => delta_e_row_scalar::<T, BD8>,
        (10, 2) => delta_e_row_scalar::<T, BD10>,
        (12, 2) => delta_e_row_scalar::<T, BD12>,
        (8, 1) => delta_e_row_scalar::<T, BD8_444>,
        (10, 1) => delta_e_row_scalar::<T, BD10_444>,
        (12, 1) => delta_e_row_scalar::<T, BD12_444>,
        _ => unreachable!(),
    }
}

pub(crate) trait Colorspace {
    const BIT_DEPTH: u32;
    const X_DECIMATION: u32;
}

struct BD8;
struct BD10;
struct BD12;

struct BD8_444;
struct BD10_444;
struct BD12_444;

impl Colorspace for BD8 {
    const BIT_DEPTH: u32 = 8;
    const X_DECIMATION: u32 = 1;
}
impl Colorspace for BD10 {
    const BIT_DEPTH: u32 = 10;
    const X_DECIMATION: u32 = 1;
}
impl Colorspace for BD12 {
    const BIT_DEPTH: u32 = 12;
    const X_DECIMATION: u32 = 1;
}
impl Colorspace for BD8_444 {
    const BIT_DEPTH: u32 = 8;
    const X_DECIMATION: u32 = 0;
}
impl Colorspace for BD10_444 {
    const BIT_DEPTH: u32 = 10;
    const X_DECIMATION: u32 = 0;
}
impl Colorspace for BD12_444 {
    const BIT_DEPTH: u32 = 12;
    const X_DECIMATION: u32 = 0;
}

pub(crate) fn delta_e_scalar<BD: Colorspace>(yuv1: (u16, u16, u16), yuv2: (u16, u16, u16)) -> f32 {
    let scale = (1 << (BD::BIT_DEPTH - 8)) as f32;
    let yuv_to_rgb = |yuv: (u16, u16, u16)| {
        // Assumes BT.709
        let y = (yuv.0 as f32 - 16. * scale) * (1. / (219. * scale));
        let u = (yuv.1 as f32 - 128. * scale) * (1. / (224. * scale));
        let v = (yuv.2 as f32 - 128. * scale) * (1. / (224. * scale));

        // [-0.804677, 1.81723]
        let r = y + 1.28033 * v;
        // [−0.316650, 1.09589]
        let g = y - 0.21482 * u - 0.38059 * v;
        // [-1.28905, 2.29781]
        let b = y + 2.12798 * u;

        (r, g, b)
    };

    let (r1, g1, b1) = yuv_to_rgb(yuv1);
    let (r2, g2, b2) = yuv_to_rgb(yuv2);
    DE2000::new(rgb_to_lab(&[r1, g1, b1]), rgb_to_lab(&[r2, g2, b2]), K_SUB)
}

pub(crate) fn delta_e_row_scalar<T: Pixel, BD: Colorspace>(
    row1: FrameRow<T>,
    row2: FrameRow<T>,
    res_row: &mut [f32],
) {
    for idx in 0..row1.y.len() {
        res_row[idx] = delta_e_scalar::<BD>(
            (
                row1.y[idx].into(),
                row1.u[idx >> BD::X_DECIMATION].into(),
                row1.v[idx >> BD::X_DECIMATION].into(),
            ),
            (
                row2.y[idx].into(),
                row2.u[idx >> BD::X_DECIMATION].into(),
                row2.v[idx >> BD::X_DECIMATION].into(),
            ),
        );
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;
