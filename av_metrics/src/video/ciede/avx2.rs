#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::video::pixel::Pixel;

use super::delta_e_row_scalar;
use super::rgbtolab::rgb_to_lab_avx2;
use super::{Colorspace, DE2000, FrameRow, K_SUB};

#[target_feature(enable = "avx2")]
fn yuv_to_rgb<BD: Colorspace>(yuv: (__m256, __m256, __m256)) -> (__m256, __m256, __m256) {
    let scale: f32 = (1 << (BD::BIT_DEPTH - 8)) as f32;
    #[target_feature(enable = "avx2")]
    fn set1(val: f32) -> __m256 {
        _mm256_set1_ps(val)
    }
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
fn delta_e_avx2<BD: Colorspace>(
    yuv1: (__m256, __m256, __m256),
    yuv2: (__m256, __m256, __m256),
    res_chunk: &mut [f32; 8],
) {
    let (r1, g1, b1) = yuv_to_rgb::<BD>(yuv1);
    let (r2, g2, b2) = yuv_to_rgb::<BD>(yuv2);

    let lab1 = rgb_to_lab_avx2(&[r1, g1, b1]);
    let lab2 = rgb_to_lab_avx2(&[r2, g2, b2]);
    for i in 0..8 {
        res_chunk[i] = DE2000::new(lab1[i], lab2[i], K_SUB);
    }
}

#[target_feature(enable = "avx2")]
pub fn delta_e_row_avx2<T: Pixel, BD: Colorspace>(
    row1: FrameRow<T>,
    row2: FrameRow<T>,
    res_row: &mut [f32],
) {
    let (r1_y_chunks, r1_y_rest) = row1.y.as_chunks::<8>();
    let (r1_u_chunks, r1_u_rest) = row1.u.as_chunks::<4>();
    let (r1_v_chunks, r1_v_rest) = row1.v.as_chunks::<4>();
    let (r2_y_chunks, r2_y_rest) = row2.y.as_chunks::<8>();
    let (r2_u_chunks, r2_u_rest) = row2.u.as_chunks::<4>();
    let (r2_v_chunks, r2_v_rest) = row2.v.as_chunks::<4>();
    let (res_chunks, res_rest) = res_row.as_chunks_mut::<8>();

    for idx in 0..r1_y_chunks.len() {
        let chunk1_y = r1_y_chunks[idx];
        let chunk1_u = r1_u_chunks[idx];
        let chunk1_v = r1_v_chunks[idx];
        let chunk2_y = r2_y_chunks[idx];
        let chunk2_u = r2_u_chunks[idx];
        let chunk2_v = r2_v_chunks[idx];
        let res_chunk = &mut res_chunks[idx];

        // Only one version should be compiled for each trait
        if BD::BIT_DEPTH == 8 {
            #[inline(always)]
            fn load_luma(chunk: &[u8; 8]) -> __m256 {
                unsafe {
                    let tmp = _mm_loadl_epi64(chunk.as_ptr().cast());
                    _mm256_cvtepi32_ps(_mm256_cvtepu8_epi32(tmp))
                }
            }

            #[inline(always)]
            fn load_chroma(chunk: [u8; 4]) -> __m256 {
                unsafe {
                    let tmp = _mm_cvtsi32_si128(i32::from_ne_bytes(chunk));
                    _mm256_cvtepi32_ps(_mm256_cvtepu8_epi32(_mm_unpacklo_epi8(tmp, tmp)))
                }
            }

            delta_e_avx2::<BD>(
                (
                    load_luma(&chunk1_y.map(|p| p.try_into().expect("Pixel is u8"))),
                    load_chroma(chunk1_u.map(|p| p.try_into().expect("Pixel is u8"))),
                    load_chroma(chunk1_v.map(|p| p.try_into().expect("Pixel is u8"))),
                ),
                (
                    load_luma(&chunk2_y.map(|p| p.try_into().expect("Pixel is u8"))),
                    load_chroma(chunk2_u.map(|p| p.try_into().expect("Pixel is u8"))),
                    load_chroma(chunk2_v.map(|p| p.try_into().expect("Pixel is u8"))),
                ),
                res_chunk,
            );
        } else {
            #[inline(always)]
            fn load_luma(chunk: &[u16; 8]) -> __m256 {
                unsafe {
                    let tmp = _mm_loadu_si128(chunk.as_ptr().cast());
                    _mm256_cvtepi32_ps(_mm256_cvtepu16_epi32(tmp))
                }
            }

            #[inline(always)]
            fn load_chroma(chunk: &[u16; 4]) -> __m256 {
                unsafe {
                    let tmp = _mm_loadl_epi64(chunk.as_ptr().cast());
                    _mm256_cvtepi32_ps(_mm256_cvtepu16_epi32(_mm_unpacklo_epi16(tmp, tmp)))
                }
            }

            delta_e_avx2::<BD>(
                (
                    load_luma(&chunk1_y.map(|p| p.into())),
                    load_chroma(&chunk1_u.map(|p| p.into())),
                    load_chroma(&chunk1_v.map(|p| p.into())),
                ),
                (
                    load_luma(&chunk2_y.map(|p| p.into())),
                    load_chroma(&chunk2_u.map(|p| p.into())),
                    load_chroma(&chunk2_v.map(|p| p.into())),
                ),
                res_chunk,
            );
        }
    }

    delta_e_row_scalar::<T, BD>(
        FrameRow {
            y: r1_y_rest,
            u: r1_u_rest,
            v: r1_v_rest,
        },
        FrameRow {
            y: r2_y_rest,
            u: r2_u_rest,
            v: r2_v_rest,
        },
        res_rest,
    );
}
