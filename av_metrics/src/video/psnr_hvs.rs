//! Peak Signal-to-Noise Ratio metric accounting for the Human Visual System.
//!
//! Humans perceive larger differences from certain factors of an image compared
//! to other factors. This metric attempts to take the human perception factor
//! into account.
//!
//! See https://en.wikipedia.org/wiki/Peak_signal-to-noise_ratio for more details.

#[cfg(feature = "decode")]
use crate::video::decode::Decoder;
use crate::video::pixel::CastFromPrimitive;
use crate::video::pixel::Pixel;
use crate::video::{FrameInfo, PlanarMetrics, PlaneData, VideoMetric};
use std::error::Error;

/// Calculates the PSNR-HVS score between two videos. Higher is better.
#[cfg(feature = "decode")]
#[inline]
pub fn calculate_video_psnr_hvs<D: Decoder>(
    decoder1: &mut D,
    decoder2: &mut D,
    frame_limit: Option<usize>,
) -> Result<PlanarMetrics, Box<dyn Error>> {
    PsnrHvs::default().process_video(decoder1, decoder2, frame_limit)
}

/// Calculates the PSNR-HVS score between two video frames. Higher is better.
#[inline]
pub fn calculate_frame_psnr_hvs<T: Pixel>(
    frame1: &FrameInfo<T>,
    frame2: &FrameInfo<T>,
) -> Result<PlanarMetrics, Box<dyn Error>> {
    let mut processor = PsnrHvs::default();
    let result = processor.process_frame(frame1, frame2)?;
    let cweight = processor.cweight.unwrap();
    Ok(PlanarMetrics {
        y: log10_convert(result.y, 1.0),
        u: log10_convert(result.u, 1.0),
        v: log10_convert(result.v, 1.0),
        avg: log10_convert(
            result.y + cweight * (result.u + result.v),
            1.0 + 2.0 * cweight,
        ),
    })
}

#[derive(Default)]
struct PsnrHvs {
    pub cweight: Option<f64>,
}

impl VideoMetric for PsnrHvs {
    type FrameResult = PlanarMetrics;
    type VideoResult = PlanarMetrics;

    /// Returns the *unweighted* scores. Depending on whether we output per-frame
    /// or per-video, these will be weighted at different points.
    fn process_frame<T: Pixel>(
        &mut self,
        frame1: &FrameInfo<T>,
        frame2: &FrameInfo<T>,
    ) -> Result<Self::FrameResult, Box<dyn Error>> {
        frame1.can_compare(&frame2)?;
        if self.cweight.is_none() {
            self.cweight = Some(frame1.chroma_sampling.get_chroma_weight());
        }

        let bit_depth = frame1.bit_depth;
        let y = calculate_plane_psnr_hvs(&frame1.planes[0], &frame2.planes[0], 0, bit_depth);
        let u = calculate_plane_psnr_hvs(&frame1.planes[1], &frame2.planes[1], 1, bit_depth);
        let v = calculate_plane_psnr_hvs(&frame1.planes[2], &frame2.planes[2], 2, bit_depth);
        Ok(PlanarMetrics {
            y,
            u,
            v,
            // field not used here
            avg: 0.,
        })
    }

    #[cfg(feature = "decode")]
    fn aggregate_frame_results(
        &self,
        metrics: &[Self::FrameResult],
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        let cweight = self.cweight.unwrap();
        let sum_y = metrics.iter().map(|m| m.y).sum::<f64>();
        let sum_u = metrics.iter().map(|m| m.u).sum::<f64>();
        let sum_v = metrics.iter().map(|m| m.v).sum::<f64>();
        Ok(PlanarMetrics {
            y: log10_convert(sum_y, 1. / metrics.len() as f64),
            u: log10_convert(sum_u, 1. / metrics.len() as f64),
            v: log10_convert(sum_v, 1. / metrics.len() as f64),
            avg: log10_convert(
                sum_y + cweight * (sum_u + sum_v),
                (1. + 2. * cweight) * 1. / metrics.len() as f64,
            ),
        })
    }
}

// Normalized inverse quantization matrix for 8x8 DCT at the point of transparency.
// This is not the JPEG based matrix from the paper,
// this one gives a slightly higher MOS agreement.
#[rustfmt::skip]
const CSF_Y: [[f64; 8]; 8] = [
    [1.6193873005, 2.2901594831, 2.08509755623, 1.48366094411, 1.00227514334, 0.678296995242, 0.466224900598, 0.3265091542],
    [2.2901594831, 1.94321815382, 2.04793073064, 1.68731108984, 1.2305666963, 0.868920337363, 0.61280991668, 0.436405793551],
    [2.08509755623, 2.04793073064, 1.34329019223, 1.09205635862, 0.875748795257, 0.670882927016, 0.501731932449, 0.372504254596],
    [1.48366094411, 1.68731108984, 1.09205635862, 0.772819797575, 0.605636379554, 0.48309405692, 0.380429446972, 0.295774038565],
    [1.00227514334, 1.2305666963, 0.875748795257, 0.605636379554, 0.448996256676, 0.352889268808, 0.283006984131, 0.226951348204],
    [0.678296995242, 0.868920337363, 0.670882927016, 0.48309405692, 0.352889268808, 0.27032073436, 0.215017739696, 0.17408067321],
    [0.466224900598, 0.61280991668, 0.501731932449, 0.380429446972, 0.283006984131, 0.215017739696, 0.168869545842, 0.136153931001],
    [0.3265091542, 0.436405793551, 0.372504254596, 0.295774038565, 0.226951348204, 0.17408067321, 0.136153931001, 0.109083846276]
];

#[rustfmt::skip]
const CSF_CB420: [[f64; 8]; 8] = [
    [1.91113096927, 2.46074210438, 1.18284184739, 1.14982565193, 1.05017074788, 0.898018824055, 0.74725392039, 0.615105596242],
    [2.46074210438, 1.58529308355, 1.21363250036, 1.38190029285, 1.33100189972, 1.17428548929, 0.996404342439, 0.830890433625],
    [1.18284184739, 1.21363250036, 0.978712413627, 1.02624506078, 1.03145147362, 0.960060382087, 0.849823426169, 0.731221236837],
    [1.14982565193, 1.38190029285, 1.02624506078, 0.861317501629, 0.801821139099, 0.751437590932, 0.685398513368, 0.608694761374],
    [1.05017074788, 1.33100189972, 1.03145147362, 0.801821139099, 0.676555426187, 0.605503172737, 0.55002013668, 0.495804539034],
    [0.898018824055, 1.17428548929, 0.960060382087, 0.751437590932, 0.605503172737, 0.514674450957, 0.454353482512, 0.407050308965],
    [0.74725392039, 0.996404342439, 0.849823426169, 0.685398513368, 0.55002013668, 0.454353482512, 0.389234902883, 0.342353999733],
    [0.615105596242, 0.830890433625, 0.731221236837, 0.608694761374, 0.495804539034, 0.407050308965, 0.342353999733, 0.295530605237]
];

#[rustfmt::skip]
const CSF_CR420: [[f64; 8]; 8] = [
    [2.03871978502, 2.62502345193, 1.26180942886, 1.11019789803, 1.01397751469, 0.867069376285, 0.721500455585, 0.593906509971],
    [2.62502345193, 1.69112867013, 1.17180569821, 1.3342742857, 1.28513006198, 1.13381474809, 0.962064122248, 0.802254508198],
    [1.26180942886, 1.17180569821, 0.944981930573, 0.990876405848, 0.995903384143, 0.926972725286, 0.820534991409, 0.706020324706],
    [1.11019789803, 1.3342742857, 0.990876405848, 0.831632933426, 0.77418706195, 0.725539939514, 0.661776842059, 0.587716619023],
    [1.01397751469, 1.28513006198, 0.995903384143, 0.77418706195, 0.653238524286, 0.584635025748, 0.531064164893, 0.478717061273],
    [0.867069376285, 1.13381474809, 0.926972725286, 0.725539939514, 0.584635025748, 0.496936637883, 0.438694579826, 0.393021669543],
    [0.721500455585, 0.962064122248, 0.820534991409, 0.661776842059, 0.531064164893, 0.438694579826, 0.375820256136, 0.330555063063],
    [0.593906509971, 0.802254508198, 0.706020324706, 0.587716619023, 0.478717061273, 0.393021669543, 0.330555063063, 0.285345396658]
];

fn calculate_plane_psnr_hvs<T: Pixel>(
    plane1: &PlaneData<T>,
    plane2: &PlaneData<T>,
    plane_idx: usize,
    bit_depth: usize,
) -> f64 {
    const STEP: usize = 7;
    let mut result = 0.0;
    let mut pixels = 0usize;
    let csf = match plane_idx {
        0 => &CSF_Y,
        1 => &CSF_CB420,
        2 => &CSF_CR420,
        _ => unreachable!(),
    };

    // In the PSNR-HVS-M paper[1] the authors describe the construction of
    // their masking table as "we have used the quantization table for the
    // color component Y of JPEG [6] that has been also obtained on the
    // basis of CSF. Note that the values in quantization table JPEG have
    // been normalized and then squared." Their CSF matrix (from PSNR-HVS)
    // was also constructed from the JPEG matrices. I can not find any obvious
    // scheme of normalizing to produce their table, but if I multiply their
    // CSF by 0.38857 and square the result I get their masking table.
    // I have no idea where this constant comes from, but deviating from it
    // too greatly hurts MOS agreement.
    //
    // [1] Nikolay Ponomarenko, Flavia Silvestri, Karen Egiazarian, Marco Carli,
    //     Jaakko Astola, Vladimir Lukin, "On between-coefficient contrast masking
    //     of DCT basis functions", CD-ROM Proceedings of the Third
    //     International Workshop on Video Processing and Quality Metrics for Consumer
    //     Electronics VPQM-07, Scottsdale, Arizona, USA, 25-26 January, 2007, 4 p.
    const CSF_MULTIPLIER: f64 = 0.3885746225901003;
    let mut mask = [[0.0; 8]; 8];
    for x in 0..8 {
        for y in 0..8 {
            mask[x][y] = (csf[x][y] * CSF_MULTIPLIER).powi(2);
        }
    }

    let height = plane1.height;
    let width = plane1.width;
    let mut p1 = [0i16; 8 * 8];
    let mut p2 = [0i16; 8 * 8];
    let mut dct_p1 = [0i32; 8 * 8];
    let mut dct_p2 = [0i32; 8 * 8];
    assert!(plane1.data.len() == width * height);
    assert!(plane2.data.len() == width * height);
    for y in (0..(height - STEP)).step_by(STEP) {
        for x in (0..(width - STEP)).step_by(STEP) {
            let mut p1_means = [0.0; 4];
            let mut p2_means = [0.0; 4];
            let mut p1_vars = [0.0; 4];
            let mut p2_vars = [0.0; 4];
            let mut p1_gmean = 0.0;
            let mut p2_gmean = 0.0;
            let mut p1_gvar = 0.0;
            let mut p2_gvar = 0.0;
            let mut p1_mask = 0.0;
            let mut p2_mask = 0.0;

            for i in 0..8 {
                for j in 0..8 {
                    p1[i * 8 + j] = i16::cast_from(plane1.data[(y + i) * width + x + j]);
                    p2[i * 8 + j] = i16::cast_from(plane2.data[(y + i) * width + x + j]);

                    let sub = ((i & 12) >> 2) + ((j & 12) >> 1);
                    p1_gmean += p1[i * 8 + j] as f64;
                    p2_gmean += p2[i * 8 + j] as f64;
                    p1_means[sub] += p1[i * 8 + j] as f64;
                    p2_means[sub] += p2[i * 8 + j] as f64;
                }
            }
            p1_gmean /= 64.0;
            p2_gmean /= 64.0;
            for i in 0..4 {
                p1_means[i] /= 16.0;
                p2_means[i] /= 16.0;
            }

            for i in 0..8 {
                for j in 0..8 {
                    let sub = ((i & 12) >> 2) + ((j & 12) >> 1);
                    p1_gvar +=
                        (p1[i * 8 + j] as f64 - p1_gmean) * (p1[i * 8 + j] as f64 - p1_gmean);
                    p2_gvar +=
                        (p2[i * 8 + j] as f64 - p2_gmean) * (p2[i * 8 + j] as f64 - p2_gmean);
                    p1_vars[sub] += (p1[i * 8 + j] as f64 - p1_means[sub])
                        * (p1[i * 8 + j] as f64 - p1_means[sub]);
                    p2_vars[sub] += (p2[i * 8 + j] as f64 - p2_means[sub])
                        * (p2[i * 8 + j] as f64 - p2_means[sub]);
                }
            }
            p1_gvar *= 64.0 / 63.0;
            p2_gvar *= 64.0 / 63.0;
            for i in 0..4 {
                p1_vars[i] *= 16.0 / 15.0;
                p2_vars[i] *= 16.0 / 15.0;
            }
            if p1_gvar > 0.0 {
                p1_gvar = p1_vars.iter().sum::<f64>() / p1_gvar;
            }
            if p2_gvar > 0.0 {
                p2_gvar = p2_vars.iter().sum::<f64>() / p2_gvar;
            }

            p1.iter().copied().enumerate().for_each(|(i, v)| {
                dct_p1[i] = v as i32;
            });
            p2.iter().copied().enumerate().for_each(|(i, v)| {
                dct_p2[i] = v as i32;
            });
            od_bin_fdct8x8(&mut dct_p1);
            od_bin_fdct8x8(&mut dct_p2);
            for i in 0..8 {
                for j in (i == 0) as usize..8 {
                    p1_mask += dct_p1[i * 8 + j].pow(2) as f64 * mask[i][j];
                    p2_mask += dct_p2[i * 8 + j].pow(2) as f64 * mask[i][j];
                }
            }
            p1_mask = (p1_mask * p1_gvar).sqrt() / 32.0;
            p2_mask = (p2_mask * p2_gvar).sqrt() / 32.0;
            if p2_mask > p1_mask {
                p1_mask = p2_mask;
            }
            for i in 0..8 {
                for j in 0..8 {
                    let mut err = (dct_p1[i * 8 + j] - dct_p2[i * 8 + j]).abs() as f64;
                    if i != 0 || j != 0 {
                        let err_mask = p1_mask / mask[i][j];
                        err = if err < err_mask { 0.0 } else { err - err_mask };
                    }
                    result += (err * csf[i][j]).powi(2);
                    pixels += 1;
                }
            }
        }
    }

    result /= pixels as f64;
    let sample_max: usize = (1 << bit_depth) - 1;
    result /= sample_max.pow(2) as f64;
    result
}

fn log10_convert(score: f64, weight: f64) -> f64 {
    10.0 * (-1.0 * (weight * score).log10())
}

const DCT_STRIDE: usize = 8;

// Based on daala's version. It is different from the 8x8 DCT we use during encoding.
fn od_bin_fdct8x8(data: &mut [i32]) {
    assert!(data.len() >= 64);
    let mut z = [0; 64];
    for i in 0..8 {
        od_bin_fdct8(&mut z[(DCT_STRIDE * i)..], &data[i..]);
    }
    for i in 0..8 {
        od_bin_fdct8(&mut data[(DCT_STRIDE * i)..], &z[i..]);
    }
}

#[allow(clippy::identity_op)]
fn od_bin_fdct8(y: &mut [i32], x: &[i32]) {
    assert!(y.len() >= 8);
    assert!(x.len() > 7 * DCT_STRIDE);
    let mut t = [0; 8];
    let mut th = [0; 8];
    // Initial permutation
    t[0] = x[0];
    t[4] = x[1 * DCT_STRIDE];
    t[2] = x[2 * DCT_STRIDE];
    t[6] = x[3 * DCT_STRIDE];
    t[7] = x[4 * DCT_STRIDE];
    t[3] = x[5 * DCT_STRIDE];
    t[5] = x[6 * DCT_STRIDE];
    t[1] = x[7 * DCT_STRIDE];
    // +1/-1 butterflies
    t[1] = t[0] - t[1];
    th[1] = od_dct_rshift(t[1], 1);
    t[0] -= th[1];
    t[4] += t[5];
    th[4] = od_dct_rshift(t[4], 1);
    t[5] -= th[4];
    t[3] = t[2] - t[3];
    t[2] -= od_dct_rshift(t[3], 1);
    t[6] += t[7];
    th[6] = od_dct_rshift(t[6], 1);
    t[7] = th[6] - t[7];
    // + Embedded 4-point type-II DCT
    t[0] += th[6];
    t[6] = t[0] - t[6];
    t[2] = th[4] - t[2];
    t[4] = t[2] - t[4];
    // |-+ Embedded 2-point type-II DCT
    t[0] -= (t[4] * 13573 + 16384) >> 15;
    t[4] += (t[0] * 11585 + 8192) >> 14;
    t[0] -= (t[4] * 13573 + 16384) >> 15;
    // |-+ Embedded 2-point type-IV DST
    t[6] -= (t[2] * 21895 + 16384) >> 15;
    t[2] += (t[6] * 15137 + 8192) >> 14;
    t[6] -= (t[2] * 21895 + 16384) >> 15;
    // + Embedded 4-point type-IV DST
    t[3] += (t[5] * 19195 + 16384) >> 15;
    t[5] += (t[3] * 11585 + 8192) >> 14;
    t[3] -= (t[5] * 7489 + 4096) >> 13;
    t[7] = od_dct_rshift(t[5], 1) - t[7];
    t[5] -= t[7];
    t[3] = th[1] - t[3];
    t[1] -= t[3];
    t[7] += (t[1] * 3227 + 16384) >> 15;
    t[1] -= (t[7] * 6393 + 16384) >> 15;
    t[7] += (t[1] * 3227 + 16384) >> 15;
    t[5] += (t[3] * 2485 + 4096) >> 13;
    t[3] -= (t[5] * 18205 + 16384) >> 15;
    t[5] += (t[3] * 2485 + 4096) >> 13;
    y[0] = t[0];
    y[1] = t[1];
    y[2] = t[2];
    y[3] = t[3];
    y[4] = t[4];
    y[5] = t[5];
    y[6] = t[6];
    y[7] = t[7];
}

/// This is the strength reduced version of `a / (1 << b)`.
/// This will not work for `b == 0`, however currently this is only used for
/// `b == 1` anyway.
#[inline(always)]
fn od_dct_rshift(a: i32, b: u32) -> i32 {
    debug_assert!(b > 0);
    debug_assert!(b <= 32);

    ((a as u32 >> (32 - b)) as i32 + a) >> b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_metric_eq;
    use std::fs::File;
    use y4m::Decoder;

    #[test]
    fn psnr_hvs_yuv420p8() {
        let mut file1 = File::open("./testfiles/yuv420p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv420p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_psnr_hvs::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(34.3227, result.y);
        assert_metric_eq(37.7400, result.u);
        assert_metric_eq(40.5570, result.v);
        assert_metric_eq(31.8676, result.avg);
    }

    #[test]
    fn psnr_hvs_yuv422p8() {
        let mut file1 = File::open("./testfiles/yuv422p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv422p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_psnr_hvs::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(45.3473, result.y);
        assert_metric_eq(46.3951, result.u);
        assert_metric_eq(45.1177, result.v);
        assert_metric_eq(39.5041, result.avg);
    }

    #[test]
    fn psnr_hvs_yuv444p8() {
        let mut file1 = File::open("./testfiles/yuv444p8_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv444p8_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_psnr_hvs::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(34.1887, result.y);
        assert_metric_eq(38.0190, result.u);
        assert_metric_eq(40.4087, result.v);
        assert_metric_eq(27.2354, result.avg);
    }

    #[test]
    fn psnr_hvs_yuv420p10() {
        let mut file1 = File::open("./testfiles/yuv420p10_input.y4m").unwrap();
        let mut dec1 = Decoder::new(&mut file1).unwrap();
        let mut file2 = File::open("./testfiles/yuv420p10_output.y4m").unwrap();
        let mut dec2 = Decoder::new(&mut file2).unwrap();
        let result = calculate_video_psnr_hvs::<_>(&mut dec1, &mut dec2, None).unwrap();
        assert_metric_eq(34.4843, result.y);
        assert_metric_eq(38.1651, result.u);
        assert_metric_eq(41.0645, result.v);
        assert_metric_eq(32.0711, result.avg);
    }
}
