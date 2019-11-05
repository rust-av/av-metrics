use av_metrics::video::*;
use clap::{App, Arg};
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;
use std::process::exit;

fn main() {
    let cli = App::new("AV Metrics")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("INPUT1")
                .help("The first input file to compare--currently supports Y4M files")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("INPUT2")
                .help("The second input file to compare--order does not matter")
                .required(true)
                .index(2),
        )
        .get_matches();
    let input1 = cli.value_of("INPUT1").unwrap();
    let input2 = cli.value_of("INPUT2").unwrap();
    let input_type1 = InputType::detect(input1);
    let input_type2 = InputType::detect(input2);
    match (input_type1, input_type2) {
        (InputType::Video(c1), InputType::Video(c2)) => {
            run_video_metrics(input1, c1, input2, c2);
        }
        (InputType::Audio(_c1), InputType::Audio(_c2)) => {
            eprintln!("No audio metrics currently implemented, exiting.");
            exit(1);
        }
        (InputType::Video(_), InputType::Audio(_)) | (InputType::Audio(_), InputType::Video(_)) => {
            eprintln!("Incompatible input files.");
            exit(1);
        }
        (InputType::Unknown, _) | (_, InputType::Unknown) => {
            eprintln!("Unsupported input format.");
            exit(1);
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum InputType {
    Video(VideoContainer),
    #[allow(dead_code)]
    Audio(AudioContainer),
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
    // TODO: Actually be generic and support more input types
    pub fn get_decoder<'d>(&self, file: &'d mut File) -> y4m::Decoder<'d, File> {
        match *self {
            VideoContainer::Y4M => y4m::Decoder::new(file).expect("Failed to read y4m file"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum AudioContainer {
    // Coming soon
}

fn run_video_metrics<P: AsRef<Path>>(
    input1: P,
    container1: VideoContainer,
    input2: P,
    container2: VideoContainer,
) {
    let mut file1 = File::open(input1).expect("Failed to open input file 1");
    let mut file2 = File::open(input2).expect("Failed to open input file 2");

    print_psnr(container1, container2, &mut file1, &mut file2);

    file1
        .seek(SeekFrom::Start(0))
        .expect("Failed to seek to start of file");
    file2
        .seek(SeekFrom::Start(0))
        .expect("Failed to seek to start of file");

    print_psnr_hvs(container1, container2, &mut file1, &mut file2);

    file1
        .seek(SeekFrom::Start(0))
        .expect("Failed to seek to start of file");
    file2
        .seek(SeekFrom::Start(0))
        .expect("Failed to seek to start of file");

    print_ssim(container1, container2, &mut file1, &mut file2);

    file1
        .seek(SeekFrom::Start(0))
        .expect("Failed to seek to start of file");
    file2
        .seek(SeekFrom::Start(0))
        .expect("Failed to seek to start of file");

    print_msssim(container1, container2, &mut file1, &mut file2);

    file1
        .seek(SeekFrom::Start(0))
        .expect("Failed to seek to start of file");
    file2
        .seek(SeekFrom::Start(0))
        .expect("Failed to seek to start of file");

    print_ciede(container1, container2, &mut file1, &mut file2);
}

fn print_psnr(
    container1: VideoContainer,
    container2: VideoContainer,
    file1: &mut File,
    file2: &mut File,
) {
    let mut dec1 = container1.get_decoder(file1);
    let mut dec2 = container2.get_decoder(file2);
    let psnr = psnr::calculate_video_psnr(&mut dec1, &mut dec2, None);
    if let Ok(psnr) = psnr {
        println!(
            "PSNR - Y: {:.4}  U: {:.4}  V: {:.4}  Avg: {:.4}",
            psnr.psnr.y, psnr.psnr.u, psnr.psnr.v, psnr.psnr.avg
        );
        println!(
            "APSNR - Y: {:.4}  U: {:.4}  V: {:.4}  Avg: {:.4}",
            psnr.apsnr.y, psnr.apsnr.u, psnr.apsnr.v, psnr.apsnr.avg
        );
    }
}

fn print_psnr_hvs(
    container1: VideoContainer,
    container2: VideoContainer,
    file1: &mut File,
    file2: &mut File,
) {
    let mut dec1 = container1.get_decoder(file1);
    let mut dec2 = container2.get_decoder(file2);
    let psnr_hvs = psnr_hvs::calculate_video_psnr_hvs(&mut dec1, &mut dec2, None);
    if let Ok(psnr_hvs) = psnr_hvs {
        println!(
            "PSNR HVS - Y: {:.4}  U: {:.4}  V: {:.4}  Avg: {:.4}",
            psnr_hvs.y, psnr_hvs.u, psnr_hvs.v, psnr_hvs.avg
        );
    }
}

fn print_ssim(
    container1: VideoContainer,
    container2: VideoContainer,
    file1: &mut File,
    file2: &mut File,
) {
    let mut dec1 = container1.get_decoder(file1);
    let mut dec2 = container2.get_decoder(file2);
    let ssim = ssim::calculate_video_ssim(&mut dec1, &mut dec2, None);
    if let Ok(ssim) = ssim {
        println!(
            "SSIM - Y: {:.4}  U: {:.4}  V: {:.4}  Avg: {:.4}",
            ssim.y, ssim.u, ssim.v, ssim.avg
        );
    }
}

fn print_msssim(
    container1: VideoContainer,
    container2: VideoContainer,
    file1: &mut File,
    file2: &mut File,
) {
    let mut dec1 = container1.get_decoder(file1);
    let mut dec2 = container2.get_decoder(file2);
    let msssim = ssim::calculate_video_msssim(&mut dec1, &mut dec2, None);
    if let Ok(msssim) = msssim {
        println!(
            "MSSSIM - Y: {:.4}  U: {:.4}  V: {:.4}  Avg: {:.4}",
            msssim.y, msssim.u, msssim.v, msssim.avg
        );
    }
}

fn print_ciede(
    container1: VideoContainer,
    container2: VideoContainer,
    file1: &mut File,
    file2: &mut File,
) {
    let mut dec1 = container1.get_decoder(file1);
    let mut dec2 = container2.get_decoder(file2);
    let ciede = ciede::calculate_video_ciede(&mut dec1, &mut dec2, None);
    if let Ok(ciede) = ciede {
        println!("CIEDE2000 - {:.4}", ciede);
    }
}
