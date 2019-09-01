use av_metrics::video;
use std::fs::File;
use y4m::Decoder;

#[inline(always)]
fn assert_metric_eq(expected: f64, value: f64) {
    assert!(
        (expected - value).abs() < 0.01,
        "Expected {}, got {}",
        expected,
        value
    );
}

#[test]
fn psnr_yuv420p8() {
    let mut file1 = File::open("./testfiles/yuv420p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv420p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_psnr::<_, u8>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(32.5281, result.psnr.y);
    assert_metric_eq(36.4083, result.psnr.u);
    assert_metric_eq(39.8238, result.psnr.v);
    assert_metric_eq(33.6861, result.psnr.avg);
    assert_metric_eq(32.5450, result.apsnr.y);
    assert_metric_eq(36.4087, result.apsnr.u);
    assert_metric_eq(39.8244, result.apsnr.v);
    assert_metric_eq(33.6995, result.apsnr.avg);
}

#[test]
fn psnr_hvs_yuv420p8() {
    let mut file1 = File::open("./testfiles/yuv420p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv420p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_psnr_hvs::<_, u8>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(34.3227, result.y);
    assert_metric_eq(37.7400, result.u);
    assert_metric_eq(40.5570, result.v);
    assert_metric_eq(31.8676, result.avg);
}

#[test]
fn ssim_yuv420p8() {
    let mut file1 = File::open("./testfiles/yuv420p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv420p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_ssim::<_, u8>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(13.2572, result.y);
    assert_metric_eq(10.8624, result.u);
    assert_metric_eq(12.8369, result.v);
    assert_metric_eq(12.6899, result.avg);
}

#[test]
fn msssim_yuv420p8() {
    let mut file1 = File::open("./testfiles/yuv420p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv420p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_msssim::<_, u8>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(18.8343, result.y);
    assert_metric_eq(16.6943, result.u);
    assert_metric_eq(18.7662, result.v);
    assert_metric_eq(18.3859, result.avg);
}

#[test]
fn ciede2000_yuv420p8() {
    let mut file1 = File::open("./testfiles/yuv420p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv420p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_ciede::<_, u8>(&mut dec1, &mut dec2, None, false).unwrap();
    assert_metric_eq(36.2821, result);
}

#[test]
fn ciede2000_yuv420p8_simd() {
    let mut file1 = File::open("./testfiles/yuv420p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv420p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_ciede::<_, u8>(&mut dec1, &mut dec2, None, true).unwrap();
    assert_metric_eq(36.2821, result);
}

#[test]
fn psnr_yuv422p8() {
    let mut file1 = File::open("./testfiles/yuv422p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv422p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_psnr::<_, u8>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(38.6740, result.psnr.y);
    assert_metric_eq(47.5219, result.psnr.u);
    assert_metric_eq(48.8615, result.psnr.v);
    assert_metric_eq(41.2190, result.psnr.avg);
    assert_metric_eq(38.6741, result.apsnr.y);
    assert_metric_eq(47.5219, result.apsnr.u);
    assert_metric_eq(48.8616, result.apsnr.v);
    assert_metric_eq(41.2191, result.apsnr.avg);
}

#[test]
fn psnr_hvs_yuv422p8() {
    let mut file1 = File::open("./testfiles/yuv422p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv422p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_psnr_hvs::<_, u8>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(45.3473, result.y);
    assert_metric_eq(46.3951, result.u);
    assert_metric_eq(45.1177, result.v);
    assert_metric_eq(39.5041, result.avg);
}

#[test]
fn ssim_yuv422p8() {
    let mut file1 = File::open("./testfiles/yuv422p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv422p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_ssim::<_, u8>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(21.1130, result.y);
    assert_metric_eq(21.9978, result.u);
    assert_metric_eq(22.7898, result.v);
    assert_metric_eq(21.6987, result.avg);
}

#[test]
fn msssim_yuv422p8() {
    let mut file1 = File::open("./testfiles/yuv422p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv422p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_msssim::<_, u8>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(28.6035, result.y);
    assert_metric_eq(28.0332, result.u);
    assert_metric_eq(28.0097, result.v);
    assert_metric_eq(28.3027, result.avg);
}

#[test]
fn psnr_yuv444p8() {
    let mut file1 = File::open("./testfiles/yuv444p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv444p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_psnr::<_, u8>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(32.4235, result.psnr.y);
    assert_metric_eq(40.1212, result.psnr.u);
    assert_metric_eq(43.1900, result.psnr.v);
    assert_metric_eq(36.2126, result.psnr.avg);
    assert_metric_eq(32.4412, result.apsnr.y);
    assert_metric_eq(40.1264, result.apsnr.u);
    assert_metric_eq(43.1943, result.apsnr.v);
    assert_metric_eq(36.2271, result.apsnr.avg);
}

#[test]
fn psnr_hvs_yuv444p8() {
    let mut file1 = File::open("./testfiles/yuv444p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv444p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_psnr_hvs::<_, u8>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(34.1887, result.y);
    assert_metric_eq(38.0190, result.u);
    assert_metric_eq(40.4087, result.v);
    assert_metric_eq(27.2354, result.avg);
}

#[test]
fn ssim_yuv444p8() {
    let mut file1 = File::open("./testfiles/yuv444p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv444p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_ssim::<_, u8>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(13.2989, result.y);
    assert_metric_eq(14.0089, result.u);
    assert_metric_eq(15.7419, result.v);
    assert_metric_eq(14.2338, result.avg);
}

#[test]
fn msssim_yuv444p8() {
    let mut file1 = File::open("./testfiles/yuv444p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv444p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_msssim::<_, u8>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(18.8897, result.y);
    assert_metric_eq(17.6092, result.u);
    assert_metric_eq(19.2732, result.v);
    assert_metric_eq(18.5308, result.avg);
}

#[test]
fn ciede2000_yuv444p8() {
    let mut file1 = File::open("./testfiles/yuv444p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv444p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_ciede::<_, u8>(&mut dec1, &mut dec2, None, false).unwrap();
    assert_metric_eq(37.5106, result);
}

#[test]
fn ciede2000_yuv444p8_simd() {
    let mut file1 = File::open("./testfiles/yuv444p8_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv444p8_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_ciede::<_, u8>(&mut dec1, &mut dec2, None, true).unwrap();
    assert_metric_eq(37.5106, result);
}

#[test]
fn psnr_yuv420p10() {
    let mut file1 = File::open("./testfiles/yuv420p10_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv420p10_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_psnr::<_, u16>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(32.5421, result.psnr.y);
    assert_metric_eq(36.4922, result.psnr.u);
    assert_metric_eq(39.8558, result.psnr.v);
    assert_metric_eq(33.7071, result.psnr.avg);
    assert_metric_eq(32.5586, result.apsnr.y);
    assert_metric_eq(36.4923, result.apsnr.u);
    assert_metric_eq(39.8563, result.apsnr.v);
    assert_metric_eq(33.7200, result.apsnr.avg);
}

#[test]
fn psnr_hvs_yuv420p10() {
    let mut file1 = File::open("./testfiles/yuv420p10_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv420p10_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_psnr_hvs::<_, u16>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(34.4843, result.y);
    assert_metric_eq(38.1651, result.u);
    assert_metric_eq(41.0645, result.v);
    assert_metric_eq(32.0711, result.avg);
}

#[test]
fn ssim_yuv420p10() {
    let mut file1 = File::open("./testfiles/yuv420p10_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv420p10_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_ssim::<_, u16>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(13.3603, result.y);
    assert_metric_eq(10.9323, result.u);
    assert_metric_eq(12.8685, result.v);
    assert_metric_eq(12.7729, result.avg);
}

#[test]
fn msssim_yuv420p10() {
    let mut file1 = File::open("./testfiles/yuv420p10_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv420p10_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_msssim::<_, u16>(&mut dec1, &mut dec2, None).unwrap();
    assert_metric_eq(19.0390, result.y);
    assert_metric_eq(16.8539, result.u);
    assert_metric_eq(18.8647, result.v);
    assert_metric_eq(18.5631, result.avg);
}

#[test]
fn ciede2000_yuv420p10() {
    let mut file1 = File::open("./testfiles/yuv420p10_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv420p10_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_ciede::<_, u16>(&mut dec1, &mut dec2, None, false).unwrap();
    assert_metric_eq(36.5142, result);
}

#[test]
fn ciede2000_yuv420p10_simd() {
    let mut file1 = File::open("./testfiles/yuv420p10_input.y4m").unwrap();
    let mut dec1 = Decoder::new(&mut file1).unwrap();
    let mut file2 = File::open("./testfiles/yuv420p10_output.y4m").unwrap();
    let mut dec2 = Decoder::new(&mut file2).unwrap();
    let result = video::calculate_video_ciede::<_, u16>(&mut dec1, &mut dec2, None, true).unwrap();
    assert_metric_eq(36.5142, result);
}
