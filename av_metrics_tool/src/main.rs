use av_metrics::video::*;
use clap::{App, Arg};
use std::collections::HashMap;
use std::fs::File;
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
        .arg(
            Arg::with_name("METRIC")
                .help("Run only one metric, instead of the entire suite")
                .long("metric")
                .takes_value(true)
                .possible_value("psnr")
                .possible_value("apsnr")
                .possible_value("psnrhvs")
                .possible_value("ssim")
                .possible_value("msssim")
                .possible_value("ciede2000"),
        )
        .arg(
            Arg::with_name("JSON")
                .help("Output results as JSON--useful for piping to other programs")
                .long("json")
                .takes_value(false),
        )
        .get_matches();
    let input1 = cli.value_of("INPUT1").unwrap();
    let input2 = cli.value_of("INPUT2").unwrap();
    let input_type1 = InputType::detect(input1);
    let input_type2 = InputType::detect(input2);
    match (input_type1, input_type2) {
        (InputType::Video(c1), InputType::Video(c2)) => {
            let serialize = cli.is_present("JSON");
            let result = match cli.value_of("METRIC") {
                None => run_video_metrics(input1, c1, input2, c2, serialize),
                Some("psnr") => run_psnr(input1, c1, input2, c2, serialize),
                Some("apsnr") => run_apsnr(input1, c1, input2, c2, serialize),
                Some("psnrhvs") => run_psnr_hvs(input1, c1, input2, c2, serialize),
                Some("ssim") => run_ssim(input1, c1, input2, c2, serialize),
                Some("msssim") => run_msssim(input1, c1, input2, c2, serialize),
                Some("ciede2000") => run_ciede2000(input1, c1, input2, c2, serialize),
                Some(_) => {
                    eprintln!("Unsupported metric, exiting.");
                    exit(1);
                }
            };
            if serialize {
                print!("{}", serde_json::to_string(&result.unwrap()).unwrap());
            }
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
    serialize: bool,
) -> Option<serde_json::Value> {
    let psnr = run_psnr(
        input1.as_ref(),
        container1,
        input2.as_ref(),
        container2,
        serialize,
    );
    let apsnr = run_apsnr(
        input1.as_ref(),
        container1,
        input2.as_ref(),
        container2,
        serialize,
    );
    let psnr_hvs = run_psnr_hvs(
        input1.as_ref(),
        container1,
        input2.as_ref(),
        container2,
        serialize,
    );
    let ssim = run_ssim(
        input1.as_ref(),
        container1,
        input2.as_ref(),
        container2,
        serialize,
    );
    let msssim = run_msssim(
        input1.as_ref(),
        container1,
        input2.as_ref(),
        container2,
        serialize,
    );
    let ciede2000 = run_ciede2000(
        input1.as_ref(),
        container1,
        input2.as_ref(),
        container2,
        serialize,
    );
    if serialize {
        let mut results = HashMap::new();
        results.insert("psnr", psnr);
        results.insert("apsnr", apsnr);
        results.insert("psnr_hvs", psnr_hvs);
        results.insert("ssim", ssim);
        results.insert("msssim", msssim);
        results.insert("ciede2000", ciede2000);
        return Some(serde_json::to_value(results).unwrap());
    }
    None
}

fn run_psnr<P: AsRef<Path>>(
    input1: P,
    container1: VideoContainer,
    input2: P,
    container2: VideoContainer,
    serialize: bool,
) -> Option<serde_json::Value> {
    let mut file1 = File::open(input1).expect("Failed to open input file 1");
    let mut file2 = File::open(input2).expect("Failed to open input file 2");
    let mut dec1 = container1.get_decoder(&mut file1);
    let mut dec2 = container2.get_decoder(&mut file2);
    let psnr = psnr::calculate_video_psnr(&mut dec1, &mut dec2, None);
    if let Ok(psnr) = psnr {
        if serialize {
            return Some(serde_json::to_value(psnr).unwrap());
        } else {
            println!(
                "PSNR - Y: {:.4}  U: {:.4}  V: {:.4}  Avg: {:.4}",
                psnr.y, psnr.u, psnr.v, psnr.avg
            );
        }
    }
    None
}

fn run_apsnr<P: AsRef<Path>>(
    input1: P,
    container1: VideoContainer,
    input2: P,
    container2: VideoContainer,
    serialize: bool,
) -> Option<serde_json::Value> {
    let mut file1 = File::open(input1).expect("Failed to open input file 1");
    let mut file2 = File::open(input2).expect("Failed to open input file 2");
    let mut dec1 = container1.get_decoder(&mut file1);
    let mut dec2 = container2.get_decoder(&mut file2);
    let apsnr = psnr::calculate_video_apsnr(&mut dec1, &mut dec2, None);
    if let Ok(apsnr) = apsnr {
        if serialize {
            return Some(serde_json::to_value(apsnr).unwrap());
        } else {
            println!(
                "APSNR - Y: {:.4}  U: {:.4}  V: {:.4}  Avg: {:.4}",
                apsnr.y, apsnr.u, apsnr.v, apsnr.avg
            );
        }
    }
    None
}

fn run_psnr_hvs<P: AsRef<Path>>(
    input1: P,
    container1: VideoContainer,
    input2: P,
    container2: VideoContainer,
    serialize: bool,
) -> Option<serde_json::Value> {
    let mut file1 = File::open(input1).expect("Failed to open input file 1");
    let mut file2 = File::open(input2).expect("Failed to open input file 2");
    let mut dec1 = container1.get_decoder(&mut file1);
    let mut dec2 = container2.get_decoder(&mut file2);
    let psnr_hvs = psnr_hvs::calculate_video_psnr_hvs(&mut dec1, &mut dec2, None);
    if let Ok(psnr_hvs) = psnr_hvs {
        if serialize {
            return Some(serde_json::to_value(psnr_hvs).unwrap());
        } else {
            println!(
                "PSNR HVS - Y: {:.4}  U: {:.4}  V: {:.4}  Avg: {:.4}",
                psnr_hvs.y, psnr_hvs.u, psnr_hvs.v, psnr_hvs.avg
            );
        }
    }
    None
}

fn run_ssim<P: AsRef<Path>>(
    input1: P,
    container1: VideoContainer,
    input2: P,
    container2: VideoContainer,
    serialize: bool,
) -> Option<serde_json::Value> {
    let mut file1 = File::open(input1).expect("Failed to open input file 1");
    let mut file2 = File::open(input2).expect("Failed to open input file 2");
    let mut dec1 = container1.get_decoder(&mut file1);
    let mut dec2 = container2.get_decoder(&mut file2);
    let ssim = ssim::calculate_video_ssim(&mut dec1, &mut dec2, None);
    if let Ok(ssim) = ssim {
        if serialize {
            return Some(serde_json::to_value(ssim).unwrap());
        } else {
            println!(
                "SSIM - Y: {:.4}  U: {:.4}  V: {:.4}  Avg: {:.4}",
                ssim.y, ssim.u, ssim.v, ssim.avg
            );
        }
    }
    None
}

fn run_msssim<P: AsRef<Path>>(
    input1: P,
    container1: VideoContainer,
    input2: P,
    container2: VideoContainer,
    serialize: bool,
) -> Option<serde_json::Value> {
    let mut file1 = File::open(input1).expect("Failed to open input file 1");
    let mut file2 = File::open(input2).expect("Failed to open input file 2");
    let mut dec1 = container1.get_decoder(&mut file1);
    let mut dec2 = container2.get_decoder(&mut file2);
    let msssim = ssim::calculate_video_msssim(&mut dec1, &mut dec2, None);
    if let Ok(msssim) = msssim {
        if serialize {
            return Some(serde_json::to_value(msssim).unwrap());
        } else {
            println!(
                "MSSSIM - Y: {:.4}  U: {:.4}  V: {:.4}  Avg: {:.4}",
                msssim.y, msssim.u, msssim.v, msssim.avg
            );
        }
    }
    None
}

fn run_ciede2000<P: AsRef<Path>>(
    input1: P,
    container1: VideoContainer,
    input2: P,
    container2: VideoContainer,
    serialize: bool,
) -> Option<serde_json::Value> {
    let mut file1 = File::open(input1).expect("Failed to open input file 1");
    let mut file2 = File::open(input2).expect("Failed to open input file 2");
    let mut dec1 = container1.get_decoder(&mut file1);
    let mut dec2 = container2.get_decoder(&mut file2);
    let ciede = ciede::calculate_video_ciede(&mut dec1, &mut dec2, None);
    if let Ok(ciede) = ciede {
        if serialize {
            return Some(serde_json::to_value(ciede).unwrap());
        } else {
            println!("CIEDE2000 - {:.4}", ciede);
        }
    }
    None
}
