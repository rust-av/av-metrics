#![allow(clippy::cast_ptr_alignment)]

//! The CIEDE2000 color difference formula.
//!
//! CIEDE2000 implementation adapted from
//! [Kyle Siefring's](https://github.com/KyleSiefring/dump_ciede2000).

#[cfg(feature = "decode")]
use crate::video::decode::Decoder;
use crate::video::pixel::{CastFromPrimitive, Pixel};
use crate::video::{FrameInfo, VideoMetric};
use std::f64;

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
#[cfg(feature = "decode")]
#[inline]
pub fn calculate_video_ciede<D: Decoder>(
    decoder1: &mut D,
    decoder2: &mut D,
    frame_limit: Option<usize>,
) -> Result<f64, Box<dyn Error>> {
    Ciede2000::default().process_video(decoder1, decoder2, frame_limit)
}

/// Calculate the CIEDE2000 metric between two video clips. Higher is better.
///
/// This version disables SIMD. It is intended to only be used
/// by tests and benchmarks.
#[cfg(all(feature = "decode", any(test, feature = "bench")))]
#[inline]
pub fn calculate_video_ciede_nosimd<D: Decoder>(
    decoder1: &mut D,
    decoder2: &mut D,
    frame_limit: Option<usize>,
) -> Result<f64, Box<dyn Error>> {
    (Ciede2000 { use_simd: false }).process_video(decoder1, decoder2, frame_limit)
}

/// Calculate the CIEDE2000 metric between two video frames. Higher is better.
#[inline]
pub fn calculate_frame_ciede<T: Pixel>(
    frame1: &FrameInfo<T>,
    frame2: &FrameInfo<T>,
) -> Result<f64, Box<dyn Error>> {
    Ciede2000::default().process_frame(frame1, frame2)
}

/// Calculate the CIEDE2000 metric between two video frames. Higher is better.
///
/// This version disables SIMD. It is intended to only be used
/// by tests and benchmarks.
#[cfg(any(test, feature = "bench"))]
#[inline]
pub fn calculate_frame_ciede_nosimd<T: Pixel>(
    frame1: &FrameInfo<T>,
    frame2: &FrameInfo<T>,
) -> Result<f64, Box<dyn Error>> {
    (Ciede2000 { use_simd: false }).process_frame(frame1, frame2)
}

struct Ciede2000 {
    use_simd: bool,
}

impl Default for Ciede2000 {
    fn default() -> Self {
        Ciede2000 { use_simd: true }
    }
}

impl VideoMetric for Ciede2000 {
    type FrameResult = f64;
    type VideoResult = f64;

    fn process_frame<T: Pixel>(
        &mut self,
        frame1: &FrameInfo<T>,
        frame2: &FrameInfo<T>,
    ) -> Result<Self::FrameResult, Box<dyn Error>> {
        frame1.can_compare(&frame2)?;

        let dec = frame1.chroma_sampling.get_decimation().unwrap_or((1, 1));
        let y_width = frame1.planes[0].cfg.width;
        let y_height = frame1.planes[0].cfg.height;
        let c_width = frame1.planes[1].cfg.width;
        let delta_e_row_fn = get_delta_e_row_fn(frame1.bit_depth, dec.0, self.use_simd);
        let mut delta_e_vec: Vec<f32> = vec![0.0; y_width * y_height];
        for i in 0..y_height {
            let y_start = i * y_width;
            let y_end = y_start + y_width;
            let c_start = (i >> dec.1) * c_width;
            let c_end = c_start + c_width;
            unsafe {
                delta_e_row_fn(
                    FrameRow {
                        y: &frame1.planes[0].data[y_start..y_end],
                        u: &frame1.planes[1].data[c_start..c_end],
                        v: &frame1.planes[2].data[c_start..c_end],
                    },
                    FrameRow {
                        y: &frame2.planes[0].data[y_start..y_end],
                        u: &frame2.planes[1].data[c_start..c_end],
                        v: &frame2.planes[2].data[c_start..c_end],
                    },
                    &mut delta_e_vec[y_start..y_end],
                );
            }
        }
        let score = 45.
            - 20.
                * (delta_e_vec.iter().map(|x| *x as f64).sum::<f64>()
                    / ((y_width * y_height) as f64))
                    .log10();
        Ok(score.min(100.))
    }

    #[cfg(feature = "decode")]
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

fn get_delta_e_row_fn<T: Pixel>(bit_depth: usize, xdec: usize, simd: bool) -> DeltaERowFn<T> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") && xdec == 1 && simd {
            return match bit_depth {
                8 => BD8::delta_e_row_avx2,
                10 => BD10::delta_e_row_avx2,
                12 => BD12::delta_e_row_avx2,
                _ => unreachable!(),
            };
        }
    }
    match (bit_depth, xdec) {
        (8, 1) => BD8::delta_e_row_scalar,
        (10, 1) => BD10::delta_e_row_scalar,
        (12, 1) => BD12::delta_e_row_scalar,
        (8, 0) => BD8_444::delta_e_row_scalar,
        (10, 0) => BD10_444::delta_e_row_scalar,
        (12, 0) => BD12_444::delta_e_row_scalar,
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

fn twice<T>(
    i: T,
) -> itertools::Interleave<<T as IntoIterator>::IntoIter, <T as IntoIterator>::IntoIter>
where
    T: IntoIterator + Clone,
{
    itertools::interleave(i.clone(), i)
}

pub(crate) trait DeltaEScalar: Colorspace {
    fn delta_e_scalar(yuv1: (u16, u16, u16), yuv2: (u16, u16, u16)) -> f32 {
        let scale = (1 << (Self::BIT_DEPTH - 8)) as f32;
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

    unsafe fn delta_e_row_scalar<T: Pixel>(
        row1: FrameRow<T>,
        row2: FrameRow<T>,
        res_row: &mut [f32],
    ) {
        if Self::X_DECIMATION == 1 {
            for (y1, u1, v1, y2, u2, v2, res) in izip!(
                row1.y,
                twice(row1.u),
                twice(row1.v),
                row2.y,
                twice(row2.u),
                twice(row2.v),
                res_row
            ) {
                *res = Self::delta_e_scalar(
                    (
                        u16::cast_from(*y1),
                        u16::cast_from(*u1),
                        u16::cast_from(*v1),
                    ),
                    (
                        u16::cast_from(*y2),
                        u16::cast_from(*u2),
                        u16::cast_from(*v2),
                    ),
                );
            }
        } else {
            for (y1, u1, v1, y2, u2, v2, res) in
                izip!(row1.y, row1.u, row1.v, row2.y, row2.u, row2.v, res_row)
            {
                *res = Self::delta_e_scalar(
                    (
                        u16::cast_from(*y1),
                        u16::cast_from(*u1),
                        u16::cast_from(*v1),
                    ),
                    (
                        u16::cast_from(*y2),
                        u16::cast_from(*u2),
                        u16::cast_from(*v2),
                    ),
                );
            }
        }
    }
}

impl DeltaEScalar for BD8 {}
impl DeltaEScalar for BD10 {}
impl DeltaEScalar for BD12 {}
impl DeltaEScalar for BD8_444 {}
impl DeltaEScalar for BD10_444 {}
impl DeltaEScalar for BD12_444 {}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use self::avx2::*;
use std::error::Error;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use super::*;

    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    pub(crate) trait DeltaEAVX2: Colorspace + DeltaEScalar {
        #[target_feature(enable = "avx2")]
        unsafe fn yuv_to_rgb(yuv: (__m256, __m256, __m256)) -> (__m256, __m256, __m256) {
            let scale: f32 = (1 << (Self::BIT_DEPTH - 8)) as f32;
            #[target_feature(enable = "avx2")]
            unsafe fn set1(val: f32) -> __m256 {
                _mm256_set1_ps(val)
            };
            let y = _mm256_mul_ps(
                _mm256_sub_ps(yuv.0, set1(16. * scale)),
                set1(1. / (219. * scale)),
            );
            let u = _mm256_mul_ps(
                _mm256_sub_ps(yuv.1, set1(128. * scale)),
                set1(1. / (224. * scale)),
            );
            let v = _mm256_mul_ps(
                _mm256_sub_ps(yuv.2, set1(128. * scale)),
                set1(1. / (224. * scale)),
            );

            let r = _mm256_add_ps(y, _mm256_mul_ps(v, set1(1.28033)));
            let g = _mm256_add_ps(
                _mm256_add_ps(y, _mm256_mul_ps(u, set1(-0.21482))),
                _mm256_mul_ps(v, set1(-0.38059)),
            );
            let b = _mm256_add_ps(y, _mm256_mul_ps(u, set1(2.12798)));

            (r, g, b)
        }

        #[target_feature(enable = "avx2")]
        unsafe fn delta_e_avx2(
            yuv1: (__m256, __m256, __m256),
            yuv2: (__m256, __m256, __m256),
            res_chunk: &mut [f32],
        ) {
            let (r1, g1, b1) = Self::yuv_to_rgb(yuv1);
            let (r2, g2, b2) = Self::yuv_to_rgb(yuv2);

            let lab1 = rgb_to_lab_avx2(&[r1, g1, b1]);
            let lab2 = rgb_to_lab_avx2(&[r2, g2, b2]);
            for i in 0..8 {
                res_chunk[i] = DE2000::new(lab1[i], lab2[i], K_SUB);
            }
        }

        #[target_feature(enable = "avx2")]
        unsafe fn delta_e_row_avx2<T: Pixel>(
            row1: FrameRow<T>,
            row2: FrameRow<T>,
            res_row: &mut [f32],
        ) {
            // Only one version should be compiled for each trait
            if Self::BIT_DEPTH == 8 {
                for (chunk1_y, chunk1_u, chunk1_v, chunk2_y, chunk2_u, chunk2_v, res_chunk) in izip!(
                    row1.y.chunks(8),
                    row1.u.chunks(4),
                    row1.v.chunks(4),
                    row2.y.chunks(8),
                    row2.u.chunks(4),
                    row2.v.chunks(4),
                    res_row.chunks_mut(8)
                ) {
                    if chunk1_y.len() == 8 {
                        #[inline(always)]
                        unsafe fn load_luma(chunk: &[u8]) -> __m256 {
                            let tmp = _mm_loadl_epi64(chunk.as_ptr() as *const _);
                            _mm256_cvtepi32_ps(_mm256_cvtepu8_epi32(tmp))
                        };

                        #[inline(always)]
                        unsafe fn load_chroma(chunk: &[u8]) -> __m256 {
                            let tmp = _mm_cvtsi32_si128(*(chunk.as_ptr() as *const i32));
                            _mm256_cvtepi32_ps(_mm256_cvtepu8_epi32(_mm_unpacklo_epi8(tmp, tmp)))
                        };

                        Self::delta_e_avx2(
                            (
                                load_luma(
                                    &chunk1_y
                                        .iter()
                                        .map(|p| u8::cast_from(*p))
                                        .collect::<Vec<_>>(),
                                ),
                                load_chroma(
                                    &chunk1_u
                                        .iter()
                                        .map(|p| u8::cast_from(*p))
                                        .collect::<Vec<_>>(),
                                ),
                                load_chroma(
                                    &chunk1_v
                                        .iter()
                                        .map(|p| u8::cast_from(*p))
                                        .collect::<Vec<_>>(),
                                ),
                            ),
                            (
                                load_luma(
                                    &chunk2_y
                                        .iter()
                                        .map(|p| u8::cast_from(*p))
                                        .collect::<Vec<_>>(),
                                ),
                                load_chroma(
                                    &chunk2_u
                                        .iter()
                                        .map(|p| u8::cast_from(*p))
                                        .collect::<Vec<_>>(),
                                ),
                                load_chroma(
                                    &chunk2_v
                                        .iter()
                                        .map(|p| u8::cast_from(*p))
                                        .collect::<Vec<_>>(),
                                ),
                            ),
                            res_chunk,
                        );
                    } else {
                        Self::delta_e_row_scalar(
                            FrameRow {
                                y: chunk1_y,
                                u: chunk1_u,
                                v: chunk1_v,
                            },
                            FrameRow {
                                y: chunk2_y,
                                u: chunk2_u,
                                v: chunk2_v,
                            },
                            res_chunk,
                        );
                    }
                }
            } else {
                for (chunk1_y, chunk1_u, chunk1_v, chunk2_y, chunk2_u, chunk2_v, res_chunk) in izip!(
                    row1.y.chunks(8),
                    row1.u.chunks(4),
                    row1.v.chunks(4),
                    row2.y.chunks(8),
                    row2.u.chunks(4),
                    row2.v.chunks(4),
                    res_row.chunks_mut(8)
                ) {
                    if chunk1_y.len() == 8 {
                        #[inline(always)]
                        unsafe fn load_luma(chunk: &[u16]) -> __m256 {
                            let tmp = _mm_loadu_si128(chunk.as_ptr() as *const _);
                            _mm256_cvtepi32_ps(_mm256_cvtepu16_epi32(tmp))
                        };

                        #[inline(always)]
                        unsafe fn load_chroma(chunk: &[u16]) -> __m256 {
                            let tmp = _mm_loadl_epi64(chunk.as_ptr() as *const _);
                            _mm256_cvtepi32_ps(_mm256_cvtepu16_epi32(_mm_unpacklo_epi16(tmp, tmp)))
                        };

                        Self::delta_e_avx2(
                            (
                                load_luma(
                                    &chunk1_y
                                        .iter()
                                        .map(|p| u16::cast_from(*p))
                                        .collect::<Vec<_>>(),
                                ),
                                load_chroma(
                                    &chunk1_u
                                        .iter()
                                        .map(|p| u16::cast_from(*p))
                                        .collect::<Vec<_>>(),
                                ),
                                load_chroma(
                                    &chunk1_v
                                        .iter()
                                        .map(|p| u16::cast_from(*p))
                                        .collect::<Vec<_>>(),
                                ),
                            ),
                            (
                                load_luma(
                                    &chunk2_y
                                        .iter()
                                        .map(|p| u16::cast_from(*p))
                                        .collect::<Vec<_>>(),
                                ),
                                load_chroma(
                                    &chunk2_u
                                        .iter()
                                        .map(|p| u16::cast_from(*p))
                                        .collect::<Vec<_>>(),
                                ),
                                load_chroma(
                                    &chunk2_v
                                        .iter()
                                        .map(|p| u16::cast_from(*p))
                                        .collect::<Vec<_>>(),
                                ),
                            ),
                            res_chunk,
                        );
                    } else {
                        Self::delta_e_row_scalar(
                            FrameRow {
                                y: chunk1_y,
                                u: chunk1_u,
                                v: chunk1_v,
                            },
                            FrameRow {
                                y: chunk2_y,
                                u: chunk2_u,
                                v: chunk2_v,
                            },
                            res_chunk,
                        );
                    }
                }
            }
        }
    }

    impl DeltaEAVX2 for BD8 {}
    impl DeltaEAVX2 for BD10 {}
    impl DeltaEAVX2 for BD12 {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_metric_eq;
    use std::fs::File;
    use y4m::Decoder;

    #[test]
    fn ciede2000_yuv420p8_nosimd() {
        let mut file1 = File::open("./testfiles/yuv420p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv420p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_ciede_nosimd::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(36.2821, result);
    }

    #[test]
    fn ciede2000_yuv420p8() {
        let mut file1 = File::open("./testfiles/yuv420p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv420p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_ciede::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(36.2821, result);
    }

    #[test]
    fn ciede2000_yuv422p8_nosimd() {
        let mut file1 = File::open("./testfiles/yuv422p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv422p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_ciede_nosimd::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(43.9618, result);
    }

    #[test]
    fn ciede2000_yuv422p8() {
        let mut file1 = File::open("./testfiles/yuv422p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv422p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_ciede::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(43.9618, result);
    }

    #[test]
    fn ciede2000_yuv444p8_nosimd() {
        let mut file1 = File::open("./testfiles/yuv444p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv444p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_ciede_nosimd::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(37.5106, result);
    }

    #[test]
    fn ciede2000_yuv444p8() {
        let mut file1 = File::open("./testfiles/yuv444p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv444p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_ciede::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(37.5106, result);
    }

    #[test]
    fn ciede2000_yuv420p10_nosimd() {
        let mut file1 = File::open("./testfiles/yuv420p10_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv420p10_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_ciede_nosimd::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(36.3691, result);
    }

    #[test]
    fn ciede2000_yuv420p10() {
        let mut file1 = File::open("./testfiles/yuv420p10_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv420p10_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_ciede::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(36.3691, result);
    }
}
