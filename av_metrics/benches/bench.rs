extern crate av_metrics;
#[macro_use]
extern crate criterion;

use av_metrics::video::ciede::{calculate_frame_ciede, calculate_frame_ciede_nosimd};
use av_metrics::video::decode::convert_chroma_data;
use av_metrics::video::psnr::calculate_frame_psnr;
use av_metrics::video::psnr_hvs::calculate_frame_psnr_hvs;
use av_metrics::video::ssim::{calculate_frame_msssim, calculate_frame_ssim};
use av_metrics::video::Frame;
use av_metrics::video::{ChromaSamplePosition, ChromaSampling, Pixel};
use criterion::Criterion;
use std::fs::File;
use y4m::Decoder as Y4MDec;

fn get_video_frame<T: Pixel>(filename: &str) -> Frame<T> {
    let mut file = File::open(filename).unwrap();
    let mut dec = Y4MDec::new(&mut file).unwrap();

    let bit_depth = dec.get_bit_depth();
    let color_space = dec.get_colorspace();
    let (chroma_sampling, chroma_sample_pos) = map_y4m_color_space(color_space);
    let width = dec.get_width();
    let height = dec.get_height();
    let bytes = dec.get_bytes_per_sample();
    let frame = dec.read_frame().unwrap();
    let mut f: Frame<T> = Frame::new_with_padding(width, height, chroma_sampling, 0);

    let (chroma_width, _) = chroma_sampling.get_chroma_dimensions(width, height);
    f.planes[0].copy_from_raw_u8(frame.get_y_plane(), width * bytes, bytes);
    convert_chroma_data(
        &mut f.planes[1],
        chroma_sample_pos,
        bit_depth,
        frame.get_u_plane(),
        chroma_width * bytes,
        bytes,
    );
    convert_chroma_data(
        &mut f.planes[2],
        chroma_sample_pos,
        bit_depth,
        frame.get_v_plane(),
        chroma_width * bytes,
        bytes,
    );

    f
}

fn map_y4m_color_space(color_space: y4m::Colorspace) -> (ChromaSampling, ChromaSamplePosition) {
    use av_metrics::video::ChromaSamplePosition::*;
    use av_metrics::video::ChromaSampling::*;
    use y4m::Colorspace::*;
    match color_space {
        Cmono => (Cs400, Unknown),
        C420jpeg => (Cs420, Bilateral),
        C420paldv => (Cs420, Interpolated),
        C420mpeg2 => (Cs420, Vertical),
        C420 | C420p10 | C420p12 => (Cs420, Colocated),
        C422 | C422p10 | C422p12 => (Cs422, Vertical),
        C444 | C444p10 | C444p12 => (Cs444, Colocated),
    }
}

pub fn psnr_benchmark(c: &mut Criterion) {
    let frame1 = get_video_frame::<u8>(&format!(
        "{}/../testfiles/yuv420p8_input.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    let frame2 = get_video_frame::<u8>(&format!(
        "{}/../testfiles/yuv420p8_output.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    c.bench_function("PSNR yuv420p8", |b| {
        b.iter(|| {
            calculate_frame_psnr(&frame1, &frame2, 8, ChromaSampling::Cs420).unwrap();
        })
    });
}

pub fn psnrhvs_benchmark(c: &mut Criterion) {
    let frame1 = get_video_frame::<u8>(&format!(
        "{}/../testfiles/yuv420p8_input.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    let frame2 = get_video_frame::<u8>(&format!(
        "{}/../testfiles/yuv420p8_output.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    c.bench_function("PSNR-HVS yuv420p8", |b| {
        b.iter(|| {
            calculate_frame_psnr_hvs(&frame1, &frame2, 8, ChromaSampling::Cs420).unwrap();
        })
    });
}

pub fn ssim_benchmark(c: &mut Criterion) {
    let frame1 = get_video_frame::<u8>(&format!(
        "{}/../testfiles/yuv420p8_input.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    let frame2 = get_video_frame::<u8>(&format!(
        "{}/../testfiles/yuv420p8_output.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    c.bench_function("SSIM yuv420p8", |b| {
        b.iter(|| {
            calculate_frame_ssim(&frame1, &frame2, 8, ChromaSampling::Cs420).unwrap();
        })
    });
}

pub fn msssim_benchmark(c: &mut Criterion) {
    let frame1 = get_video_frame::<u8>(&format!(
        "{}/../testfiles/yuv420p8_input.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    let frame2 = get_video_frame::<u8>(&format!(
        "{}/../testfiles/yuv420p8_output.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    c.bench_function("MSSSIM yuv420p8", |b| {
        b.iter(|| {
            calculate_frame_msssim(&frame1, &frame2, 8, ChromaSampling::Cs420).unwrap();
        })
    });
}

pub fn ciede2000_nosimd_benchmark(c: &mut Criterion) {
    let frame1 = get_video_frame::<u8>(&format!(
        "{}/../testfiles/yuv420p8_input.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    let frame2 = get_video_frame::<u8>(&format!(
        "{}/../testfiles/yuv420p8_output.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    c.bench_function("CIEDE2000 yuv420p8 nosimd", |b| {
        b.iter(|| {
            calculate_frame_ciede_nosimd(&frame1, &frame2, 8, ChromaSampling::Cs420).unwrap();
        })
    });
}

pub fn ciede2000_simd_benchmark(c: &mut Criterion) {
    let frame1 = get_video_frame::<u8>(&format!(
        "{}/../testfiles/yuv420p8_input.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    let frame2 = get_video_frame::<u8>(&format!(
        "{}/../testfiles/yuv420p8_output.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    c.bench_function("CIEDE2000 yuv420p8", |b| {
        b.iter(|| {
            calculate_frame_ciede(&frame1, &frame2, 8, ChromaSampling::Cs420).unwrap();
        })
    });
}

pub fn psnr_10bit_benchmark(c: &mut Criterion) {
    let frame1 = get_video_frame::<u16>(&format!(
        "{}/../testfiles/yuv420p10_input.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    let frame2 = get_video_frame::<u16>(&format!(
        "{}/../testfiles/yuv420p10_output.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    c.bench_function("PSNR yuv420p10", |b| {
        b.iter(|| {
            calculate_frame_psnr(&frame1, &frame2, 10, ChromaSampling::Cs420).unwrap();
        })
    });
}

pub fn psnrhvs_10bit_benchmark(c: &mut Criterion) {
    let frame1 = get_video_frame::<u16>(&format!(
        "{}/../testfiles/yuv420p10_input.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    let frame2 = get_video_frame::<u16>(&format!(
        "{}/../testfiles/yuv420p10_output.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    c.bench_function("PSNR-HVS yuv420p10", |b| {
        b.iter(|| {
            calculate_frame_psnr_hvs(&frame1, &frame2, 10, ChromaSampling::Cs420).unwrap();
        })
    });
}

pub fn ssim_10bit_benchmark(c: &mut Criterion) {
    let frame1 = get_video_frame::<u16>(&format!(
        "{}/../testfiles/yuv420p10_input.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    let frame2 = get_video_frame::<u16>(&format!(
        "{}/../testfiles/yuv420p10_output.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    c.bench_function("SSIM yuv420p10", |b| {
        b.iter(|| {
            calculate_frame_ssim(&frame1, &frame2, 10, ChromaSampling::Cs420).unwrap();
        })
    });
}

pub fn msssim_10bit_benchmark(c: &mut Criterion) {
    let frame1 = get_video_frame::<u16>(&format!(
        "{}/../testfiles/yuv420p10_input.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    let frame2 = get_video_frame::<u16>(&format!(
        "{}/../testfiles/yuv420p10_output.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    c.bench_function("MSSSIM yuv420p10", |b| {
        b.iter(|| {
            calculate_frame_msssim(&frame1, &frame2, 10, ChromaSampling::Cs420).unwrap();
        })
    });
}

pub fn ciede2000_nosimd_10bit_benchmark(c: &mut Criterion) {
    let frame1 = get_video_frame::<u16>(&format!(
        "{}/../testfiles/yuv420p10_input.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    let frame2 = get_video_frame::<u16>(&format!(
        "{}/../testfiles/yuv420p10_output.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    c.bench_function("CIEDE2000 yuv420p10 nosimd", |b| {
        b.iter(|| {
            calculate_frame_ciede_nosimd(&frame1, &frame2, 10, ChromaSampling::Cs420).unwrap();
        })
    });
}

pub fn ciede2000_simd_10bit_benchmark(c: &mut Criterion) {
    let frame1 = get_video_frame::<u16>(&format!(
        "{}/../testfiles/yuv420p10_input.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    let frame2 = get_video_frame::<u16>(&format!(
        "{}/../testfiles/yuv420p10_output.y4m",
        env!("CARGO_MANIFEST_DIR")
    ));
    c.bench_function("CIEDE2000 yuv420p10", |b| {
        b.iter(|| {
            calculate_frame_ciede(&frame1, &frame2, 10, ChromaSampling::Cs420).unwrap();
        })
    });
}

criterion_group!(
    benches,
    psnr_benchmark,
    psnrhvs_benchmark,
    ssim_benchmark,
    msssim_benchmark,
    ciede2000_nosimd_benchmark,
    ciede2000_simd_benchmark,
    psnr_10bit_benchmark,
    psnrhvs_10bit_benchmark,
    ssim_10bit_benchmark,
    msssim_10bit_benchmark,
    ciede2000_nosimd_10bit_benchmark,
    ciede2000_simd_10bit_benchmark
);
criterion_main!(benches);
