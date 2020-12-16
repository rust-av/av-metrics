#[cfg(test)]
mod tests {
    use av_metrics::video::ciede::{calculate_video_ciede, calculate_video_ciede_nosimd};
    use av_metrics::video::psnr::{calculate_video_apsnr, calculate_video_psnr};
    use av_metrics::video::psnr_hvs::calculate_video_psnr_hvs;
    use av_metrics::video::ssim::{calculate_video_msssim, calculate_video_ssim};
    #[cfg(feature = "ffmpeg")]
    use av_metrics_decoders::FfmpegDecoder;
    #[cfg(not(feature = "ffmpeg"))]
    use av_metrics_decoders::Y4MDecoder;
    use std::path::Path;

    #[cfg(not(feature = "ffmpeg"))]
    fn get_decoder<P: AsRef<Path>>(input: P) -> Result<Y4MDecoder, String> {
        Y4MDecoder::new(input)
    }

    #[cfg(feature = "ffmpeg")]
    fn get_decoder<P: AsRef<Path>>(input: P) -> Result<FfmpegDecoder, String> {
        FfmpegDecoder::new(input)
    }

    #[test]
    fn psnr_yuv420p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv420p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv420p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_psnr(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(32.5281, result.y);
        assert_metric_eq(36.4083, result.u);
        assert_metric_eq(39.8238, result.v);
        assert_metric_eq(33.6861, result.avg);
    }

    #[test]
    fn psnr_yuv422p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv422p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv422p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_psnr(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(38.6740, result.y);
        assert_metric_eq(47.5219, result.u);
        assert_metric_eq(48.8615, result.v);
        assert_metric_eq(41.2190, result.avg);
    }

    #[test]
    fn psnr_yuv444p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv444p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv444p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_psnr(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(32.4235, result.y);
        assert_metric_eq(40.1212, result.u);
        assert_metric_eq(43.1900, result.v);
        assert_metric_eq(36.2126, result.avg);
    }

    #[test]
    fn psnr_yuv420p10() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv420p10_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv420p10_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_psnr(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(32.5421, result.y);
        assert_metric_eq(36.4922, result.u);
        assert_metric_eq(39.8558, result.v);
        assert_metric_eq(33.7071, result.avg);
    }

    #[test]
    fn apsnr_yuv420p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv420p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv420p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_apsnr(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(32.5450, result.y);
        assert_metric_eq(36.4087, result.u);
        assert_metric_eq(39.8244, result.v);
        assert_metric_eq(33.6995, result.avg);
    }

    #[test]
    fn apsnr_yuv422p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv422p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv422p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_apsnr(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(38.6741, result.y);
        assert_metric_eq(47.5219, result.u);
        assert_metric_eq(48.8616, result.v);
        assert_metric_eq(41.2191, result.avg);
    }

    #[test]
    fn apsnr_yuv444p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv444p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv444p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_apsnr(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(32.4412, result.y);
        assert_metric_eq(40.1264, result.u);
        assert_metric_eq(43.1943, result.v);
        assert_metric_eq(36.2271, result.avg);
    }

    #[test]
    fn apsnr_yuv420p10() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv420p10_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv420p10_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_apsnr(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(32.5586, result.y);
        assert_metric_eq(36.4923, result.u);
        assert_metric_eq(39.8563, result.v);
        assert_metric_eq(33.7200, result.avg);
    }

    #[test]
    fn psnr_hvs_yuv420p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv420p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv420p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_psnr_hvs(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(34.3227, result.y);
        assert_metric_eq(37.7400, result.u);
        assert_metric_eq(40.5570, result.v);
        assert_metric_eq(31.8676, result.avg);
    }

    #[test]
    fn psnr_hvs_yuv422p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv422p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv422p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_psnr_hvs(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(45.3473, result.y);
        assert_metric_eq(46.3951, result.u);
        assert_metric_eq(45.1177, result.v);
        assert_metric_eq(39.5041, result.avg);
    }

    #[test]
    fn psnr_hvs_yuv444p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv444p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv444p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_psnr_hvs(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(34.1887, result.y);
        assert_metric_eq(38.0190, result.u);
        assert_metric_eq(40.4087, result.v);
        assert_metric_eq(27.2354, result.avg);
    }

    #[test]
    fn psnr_hvs_yuv420p10() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv420p10_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv420p10_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_psnr_hvs(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(34.4843, result.y);
        assert_metric_eq(38.1651, result.u);
        assert_metric_eq(41.0645, result.v);
        assert_metric_eq(32.0711, result.avg);
    }

    #[test]
    fn ssim_yuv420p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv420p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv420p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_ssim(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(13.2572, result.y);
        assert_metric_eq(10.8624, result.u);
        assert_metric_eq(12.8369, result.v);
        assert_metric_eq(12.6899, result.avg);
    }

    #[test]
    fn msssim_yuv420p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv420p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv420p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_msssim(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(18.8343, result.y);
        assert_metric_eq(16.6943, result.u);
        assert_metric_eq(18.7662, result.v);
        assert_metric_eq(18.3859, result.avg);
    }

    #[test]
    fn ssim_yuv422p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv422p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv422p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_ssim(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(21.1130, result.y);
        assert_metric_eq(21.9978, result.u);
        assert_metric_eq(22.7898, result.v);
        assert_metric_eq(21.6987, result.avg);
    }

    #[test]
    fn msssim_yuv422p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv422p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv422p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_msssim(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(28.6035, result.y);
        assert_metric_eq(28.0332, result.u);
        assert_metric_eq(28.0097, result.v);
        assert_metric_eq(28.3027, result.avg);
    }

    #[test]
    fn ssim_yuv444p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv444p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv444p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_ssim(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(13.2989, result.y);
        assert_metric_eq(14.0089, result.u);
        assert_metric_eq(15.7419, result.v);
        assert_metric_eq(14.2338, result.avg);
    }

    #[test]
    fn msssim_yuv444p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv444p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv444p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_msssim(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(18.8897, result.y);
        assert_metric_eq(17.6092, result.u);
        assert_metric_eq(19.2732, result.v);
        assert_metric_eq(18.5308, result.avg);
    }

    #[test]
    fn ssim_yuv420p10() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv420p10_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv420p10_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_ssim(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(13.3603, result.y);
        assert_metric_eq(10.9323, result.u);
        assert_metric_eq(12.8685, result.v);
        assert_metric_eq(12.7729, result.avg);
    }

    #[test]
    fn msssim_yuv420p10() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv420p10_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv420p10_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_msssim(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(19.0390, result.y);
        assert_metric_eq(16.8539, result.u);
        assert_metric_eq(18.8647, result.v);
        assert_metric_eq(18.5631, result.avg);
    }

    #[test]
    fn ciede2000_yuv420p8_nosimd() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv420p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv420p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_ciede_nosimd(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(36.2821, result);
    }

    #[test]
    fn ciede2000_yuv420p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv420p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv420p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_ciede(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(36.2821, result);
    }

    #[test]
    fn ciede2000_yuv422p8_nosimd() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv422p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv422p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_ciede_nosimd(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(43.9618, result);
    }

    #[test]
    fn ciede2000_yuv422p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv422p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv422p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_ciede(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(43.9618, result);
    }

    #[test]
    fn ciede2000_yuv444p8_nosimd() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv444p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv444p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_ciede_nosimd(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(37.5106, result);
    }

    #[test]
    fn ciede2000_yuv444p8() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv444p8_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv444p8_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_ciede(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(37.5106, result);
    }

    #[test]
    fn ciede2000_yuv420p10_nosimd() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv420p10_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv420p10_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_ciede_nosimd(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(36.3691, result);
    }

    #[test]
    fn ciede2000_yuv420p10() {
        let mut dec1 = get_decoder(&format!(
            "{}/../testfiles/yuv420p10_input.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let mut dec2 = get_decoder(&format!(
            "{}/../testfiles/yuv420p10_output.y4m",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let result = calculate_video_ciede(&mut dec1, &mut dec2, None, |_| ()).unwrap();
        assert_metric_eq(36.3691, result);
    }

    fn assert_metric_eq(expected: f64, value: f64) {
        assert!(
            (expected - value).abs() < 0.01,
            "Expected {}, got {}",
            expected,
            value
        );
    }
}
