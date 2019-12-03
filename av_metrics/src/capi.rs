#![allow(dead_code)]

extern crate libc;

use libc::c_char;
use std::ffi::CStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::ptr::null;

use crate::video::*;

#[derive(Debug, Clone, Copy)]
enum InputType {
    Video(VideoContainer),
    Unknown,
}

impl InputType {
    pub fn detect<P: AsRef<Path>>(filename: P) -> Self {
        let ext = filename
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        match ext.to_lowercase().as_str() {
            "y4m" => InputType::Video(VideoContainer::Y4M),
            _ => InputType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum VideoContainer {
    Y4M,
}

impl VideoContainer {
    pub fn get_decoder<'d>(&self, file: &'d mut File, metric: &str) -> y4m::Decoder<'d, File> {
        match *self {
            VideoContainer::Y4M => y4m::Decoder::new(file)
                .expect(&("Failed to decode the ".to_owned() + metric + " y4m file")),
        }
    }
}

#[inline(always)]
fn convert_c_string_into_path(c_buf: *const c_char) -> PathBuf {
    let c_str = unsafe { CStr::from_ptr(c_buf) };
    Path::new(c_str.to_str().unwrap()).to_path_buf()
}

fn run_metric(
    path1: *const c_char,
    path2: *const c_char,
    frame_limit: usize,
    metric: &str,
    is_frame: bool,
) -> (*const MetricContext, f64) {
    if path1.is_null() || path2.is_null() {
        return (null(), -1.0);
    }

    let path1 = convert_c_string_into_path(path1);
    let path2 = convert_c_string_into_path(path2);

    let input_type1 = InputType::detect(&path1);
    let input_type2 = InputType::detect(&path2);

    match (input_type1, input_type2) {
        (InputType::Video(c1), InputType::Video(c2)) => {
            if is_frame {
                return run_frame_metric(path1, c1, path2, c2, frame_limit, metric);
            } else {
                return run_video_metric(path1, c1, path2, c2, frame_limit, metric);
            }
        }
        (InputType::Unknown, _) | (_, InputType::Unknown) => {}
    }

    (null(), -1.0)
}

fn run_video_metric<P: AsRef<Path>>(
    path1: P,
    container1: VideoContainer,
    path2: P,
    container2: VideoContainer,
    frame_limit: usize,
    metric: &str,
) -> (*const MetricContext, f64) {
    let mut file1 =
        File::open(path1).expect(&("Error opening the first ".to_owned() + metric + " video"));
    let mut file2 =
        File::open(path2).expect(&("Error opening the second ".to_owned() + metric + " video"));

    let mut dec1 = container1.get_decoder(&mut file1, &("first".to_owned() + metric));
    let mut dec2 = container2.get_decoder(&mut file2, &("second".to_owned() + metric));

    let mut limit: Option<usize> = None;
    if frame_limit > 0 {
        limit = Some(frame_limit);
    }
    if metric == "ciede" {
        let val = ciede::calculate_video_ciede(&mut dec1, &mut dec2, limit);
        if let Ok(metric) = val {
            return (null(), metric);
        }
    }
    let val = match metric {
        "psnr" => psnr::calculate_video_psnr(&mut dec1, &mut dec2, limit),
        "apsnr" => psnr::calculate_video_apsnr(&mut dec1, &mut dec2, limit),
        "psnr_hvs" => psnr_hvs::calculate_video_psnr_hvs(&mut dec1, &mut dec2, limit),
        "ssim" => ssim::calculate_video_ssim(&mut dec1, &mut dec2, limit),
        _ => ssim::calculate_video_msssim(&mut dec1, &mut dec2, limit),
    };
    if let Ok(metric) = val {
        let ctx = MetricContext {
            y: metric.y,
            u: metric.u,
            v: metric.v,
            avg: metric.avg,
        };
        let boxed = Box::new(ctx);
        return (Box::into_raw(boxed), 0.0);
    }
    (null(), -1.0)
}

fn run_frame_metric<P: AsRef<Path>>(
    path1: P,
    container1: VideoContainer,
    path2: P,
    container2: VideoContainer,
    frame_number: usize,
    metric: &str,
) -> (*const MetricContext, f64) {
    let mut file1 =
        File::open(path1).expect(&("Error opening the first ".to_owned() + metric + " video"));
    let mut file2 =
        File::open(path2).expect(&("Error opening the second ".to_owned() + metric + " video"));

    let mut dec1 = container1.get_decoder(&mut file1, &("first".to_owned() + metric));
    let mut dec2 = container2.get_decoder(&mut file2, &("second".to_owned() + metric));

    if dec1.get_bit_depth() > 8 {
        let frame1 = dec1.read_specific_frame::<u16>(frame_number);
        let frame2 = dec2.read_specific_frame::<u16>(frame_number);
        if let Ok(frame1) = frame1 {
            if let Ok(frame2) = frame2 {
                if metric == "ciede" {
                    let val = ciede::calculate_frame_ciede(&frame1, &frame2);
                    if let Ok(metric) = val {
                        return (null(), metric);
                    }
                }
                let val = match metric {
                    "psnr" => psnr::calculate_frame_psnr(&frame1, &frame2),
                    "psnr_hvs" => psnr_hvs::calculate_frame_psnr_hvs(&frame1, &frame2),
                    "ssim" => ssim::calculate_frame_ssim(&frame1, &frame2),
                    _ => ssim::calculate_frame_msssim(&frame1, &frame2),
                };
                if let Ok(metric) = val {
                    let ctx = MetricContext {
                        y: metric.y,
                        u: metric.u,
                        v: metric.v,
                        avg: metric.avg,
                    };
                    let boxed = Box::new(ctx);
                    return (Box::into_raw(boxed), 0.0);
                }
            }
        }
    } else {
        let frame1 = dec1.read_specific_frame::<u8>(frame_number);
        let frame2 = dec2.read_specific_frame::<u8>(frame_number);
        if let Ok(frame1) = frame1 {
            if let Ok(frame2) = frame2 {
                if metric == "ciede" {
                    let val = ciede::calculate_frame_ciede(&frame1, &frame2);
                    if let Ok(metric) = val {
                        return (null(), metric);
                    }
                }
                let val = match metric {
                    "psnr" => psnr::calculate_frame_psnr(&frame1, &frame2),
                    "psnr_hvs" => psnr_hvs::calculate_frame_psnr_hvs(&frame1, &frame2),
                    "ssim" => ssim::calculate_frame_ssim(&frame1, &frame2),
                    _ => ssim::calculate_frame_msssim(&frame1, &frame2),
                };
                if let Ok(metric) = val {
                    let ctx = MetricContext {
                        y: metric.y,
                        u: metric.u,
                        v: metric.v,
                        avg: metric.avg,
                    };
                    let boxed = Box::new(ctx);
                    return (Box::into_raw(boxed), 0.0);
                }
            }
        }
    }
    (null(), -1.0)
}

/// Metric Context
///
/// This struct contains the data returned by a metric
#[repr(C)]
pub struct MetricContext {
    /// Metric value for the Y plane.
    pub y: f64,
    /// Metric value for the U/Cb plane.
    pub u: f64,
    /// Metric value for the V/Cb plane.
    pub v: f64,
    /// Weighted average of the three planes.
    pub avg: f64,
}

/// Calculate the `psnr` metric between two videos
///
/// Returns either `NULL` or a newly allocated `MetricContext`
#[no_mangle]
pub unsafe extern fn calculate_video_psnr(
    video1_path: *const c_char,
    video2_path: *const c_char,
    frame_limit: usize,
) -> *const MetricContext {
    let (metric, _) = run_metric(video1_path, video2_path, frame_limit, "psnr", false);

    metric
}

/// Calculate the `apsnr` metric between two videos
///
/// Returns either `NULL` or a newly allocated `MetricContext`
#[no_mangle]
pub unsafe extern fn calculate_video_apsnr(
    video1_path: *const c_char,
    video2_path: *const c_char,
    frame_limit: usize,
) -> *const MetricContext {
    let (metric, _) = run_metric(video1_path, video2_path, frame_limit, "apsnr", false);

    metric
}

/// Calculate the `psnr_hvs` metric between two videos
///
/// Returns either `NULL` or a newly allocated `MetricContext`
#[no_mangle]
pub unsafe extern fn calculate_video_psnr_hvs(
    video1_path: *const c_char,
    video2_path: *const c_char,
    frame_limit: usize,
) -> *const MetricContext {
    let (metric, _) = run_metric(video1_path, video2_path, frame_limit, "psnr_hvs", false);

    metric
}

/// Calculate the `ssim` metric between two videos
///
/// Returns either `NULL` or a newly allocated `MetricContext`
#[no_mangle]
pub unsafe extern fn calculate_video_ssim(
    video1_path: *const c_char,
    video2_path: *const c_char,
    frame_limit: usize,
) -> *const MetricContext {
    let (metric, _) = run_metric(video1_path, video2_path, frame_limit, "ssim", false);

    metric
}

/// Calculate the `msssim` metric between two videos
///
/// Returns either `NULL` or a newly allocated `MetricContext`
#[no_mangle]
pub unsafe extern fn calculate_video_msssim(
    video1_path: *const c_char,
    video2_path: *const c_char,
    frame_limit: usize,
) -> *const MetricContext {
    let (metric, _) = run_metric(video1_path, video2_path, frame_limit, "msssim", false);

    metric
}

/// Calculate the `ciede` metric between two videos
///
/// Returns the correct `ciede` value or `-1` on errors
#[no_mangle]
pub unsafe extern fn calculate_video_ciede(
    video1_path: *const c_char,
    video2_path: *const c_char,
    frame_limit: usize,
) -> f64 {
    let (_, value) = run_metric(video1_path, video2_path, frame_limit, "ciede", false);

    value
}

/// Calculate the `psnr` metric between two frames
///
/// Returns either `NULL` or a newly allocated `MetricContext`
#[no_mangle]
pub unsafe extern fn calculate_frame_psnr(
    video1_path: *const c_char,
    video2_path: *const c_char,
    frame_number: usize,
) -> *const MetricContext {
    let (metric, _) = run_metric(video1_path, video2_path, frame_number, "psnr", true);
    metric
}

/// Calculate the `psnr_hvs` metric between two frames
///
/// Returns either `NULL` or a newly allocated `MetricContext`
#[no_mangle]
pub unsafe extern fn calculate_frame_psnr_hvs(
    video1_path: *const c_char,
    video2_path: *const c_char,
    frame_number: usize,
) -> *const MetricContext {
    let (metric, _) = run_metric(video1_path, video2_path, frame_number, "psnr_hvs", true);
    metric
}

/// Calculate the `ssim` metric between two frames
///
/// Returns either `NULL` or a newly allocated `MetricContext`
#[no_mangle]
pub unsafe extern fn calculate_frame_ssim(
    video1_path: *const c_char,
    video2_path: *const c_char,
    frame_number: usize,
) -> *const MetricContext {
    let (metric, _) = run_metric(video1_path, video2_path, frame_number, "ssim", true);
    metric
}

/// Calculate the `msssim` metric between two frames
///
/// Returns either `NULL` or a newly allocated `MetricContext`
#[no_mangle]
pub unsafe extern fn calculate_frame_msssim(
    video1_path: *const c_char,
    video2_path: *const c_char,
    frame_number: usize,
) -> *const MetricContext {
    let (metric, _) = run_metric(video1_path, video2_path, frame_number, "msssim", true);
    metric
}

/// Calculate the `ciede` metric between two frames
///
/// Returns the correct `ciede` value or `-1` on errors
#[no_mangle]
pub unsafe extern fn calculate_frame_ciede(
    video1_path: *const c_char,
    video2_path: *const c_char,
    frame_number: usize,
) -> f64 {
    let (_, value) = run_metric(video1_path, video2_path, frame_number, "ciede", true);
    value
}

/// Drop the metric context
///
/// This function drops the context and free the memory
#[no_mangle]
pub unsafe extern fn drop_context(ctx: *const MetricContext) {
    std::mem::drop(Box::from_raw(ctx as *mut MetricContext));
}
