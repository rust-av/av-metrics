use av_metrics::video::*;
use clap::{App, Arg};
use console::style;
use maplit::hashmap;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
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
            run_video_metrics(
                input1,
                c1,
                input2,
                c2,
                cli.is_present("JSON"),
                cli.value_of("METRIC"),
            );
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
    metric: Option<&str>,
) {
    let mut results = HashMap::new();
    if !serialize {
        match metric {
            Some(metr) => println!(
                "  {} metric for: {} using the {} system...",
                style("Computing").yellow(),
                style(metr).cyan(),
                style("YUV/YCbCr").magenta()
            ),
            None => println!(
                "  {} metrics for: {}, {}, {}, {}, {}, {} using the {} system...",
                style("Computing").yellow(),
                style("PSNR").cyan(),
                style("APSNR").cyan(),
                style("PSNR-HVS").cyan(),
                style("SSIM").cyan(),
                style("MSSIM").cyan(),
                style("CIEDE2000").cyan(),
                style("YUV/YCbCr").magenta()
            ),
        }

        println!(
            "    {} for comparing {} to {}: \n",
            style("Results").yellow(),
            style(input1.as_ref().display()).italic().cyan(),
            style(input2.as_ref().display()).italic().cyan()
        );
    }

    if metric.is_none() || metric == Some("psnr") {
        let psnr = Psnr::run(
            input1.as_ref(),
            container1,
            input2.as_ref(),
            container2,
            serialize,
        );
        if serialize {
            results.insert("psnr", hashmap! {"result" => psnr});
        }
    }

    if metric.is_none() || metric == Some("apsnr") {
        let apsnr = APsnr::run(
            input1.as_ref(),
            container1,
            input2.as_ref(),
            container2,
            serialize,
        );
        if serialize {
            results.insert("apsnr", hashmap! {"result" => apsnr});
        }
    }

    if metric.is_none() || metric == Some("psnrhvs") {
        let psnrhvs = PsnrHvs::run(
            input1.as_ref(),
            container1,
            input2.as_ref(),
            container2,
            serialize,
        );
        if serialize {
            results.insert("psnrhvs", hashmap! {"result" => psnrhvs});
        }
    }

    if metric.is_none() || metric == Some("ssim") {
        let ssim = Ssim::run(
            input1.as_ref(),
            container1,
            input2.as_ref(),
            container2,
            serialize,
        );
        if serialize {
            results.insert("ssim", hashmap! {"result" => ssim});
        }
    }

    if metric.is_none() || metric == Some("msssim") {
        let msssim = MsSsim::run(
            input1.as_ref(),
            container1,
            input2.as_ref(),
            container2,
            serialize,
        );
        if serialize {
            results.insert("msssim", hashmap! {"result" => msssim});
        }
    }

    if metric.is_none() || metric == Some("ciede2000") {
        let ciede2000 = Ciede2000::run(
            input1.as_ref(),
            container1,
            input2.as_ref(),
            container2,
            serialize,
        );
        if serialize {
            results.insert("ciede2000", hashmap! {"result" => ciede2000});
        }
    }

    if serialize {
        print!("{}", serde_json::to_string(&results).unwrap());
    }
}

trait CliMetric {
    type VideoResult: Serialize;

    fn run<P: AsRef<Path>>(
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
        let result = Self::calculate_video_metric(&mut dec1, &mut dec2);
        if let Ok(result) = result {
            if serialize {
                return Some(serde_json::to_value(result).unwrap());
            } else {
                Self::print_results(result);
            }
        }
        None
    }

    fn calculate_video_metric<D: Decoder>(
        dec1: &mut D,
        dec2: &mut D,
    ) -> Result<Self::VideoResult, Box<dyn Error>>;
    fn print_results(result: Self::VideoResult);
}

struct Psnr;

impl CliMetric for Psnr {
    type VideoResult = PlanarMetrics;

    fn calculate_video_metric<D: Decoder>(
        dec1: &mut D,
        dec2: &mut D,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        psnr::calculate_video_psnr(dec1, dec2, None)
    }

    fn print_results(result: Self::VideoResult) {
        println!(
            "     {:<10} →  Y: {:<8.4} U/Cb: {:<8.4} V/Cr: {:<8.4} Avg value: {:<8.4}",
            style("PSNR").cyan(),
            result.y,
            result.u,
            result.v,
            result.avg
        );
    }
}

struct APsnr;

impl CliMetric for APsnr {
    type VideoResult = PlanarMetrics;

    fn calculate_video_metric<D: Decoder>(
        dec1: &mut D,
        dec2: &mut D,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        psnr::calculate_video_apsnr(dec1, dec2, None)
    }

    fn print_results(result: Self::VideoResult) {
        println!(
            "     {:<10} →  Y: {:<8.4} U/Cb: {:<8.4} V/Cr: {:<8.4} Avg value: {:<8.4}",
            style("APSNR").cyan(),
            result.y,
            result.u,
            result.v,
            result.avg
        );
    }
}

struct PsnrHvs;

impl CliMetric for PsnrHvs {
    type VideoResult = PlanarMetrics;

    fn calculate_video_metric<D: Decoder>(
        dec1: &mut D,
        dec2: &mut D,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        psnr_hvs::calculate_video_psnr_hvs(dec1, dec2, None)
    }

    fn print_results(result: Self::VideoResult) {
        println!(
            "     {:<10} →  Y: {:<8.4} U/Cb: {:<8.4} V/Cr: {:<8.4} Avg value: {:<8.4}",
            style("PSNR HVS").cyan(),
            result.y,
            result.u,
            result.v,
            result.avg
        );
    }
}

struct Ssim;

impl CliMetric for Ssim {
    type VideoResult = PlanarMetrics;

    fn calculate_video_metric<D: Decoder>(
        dec1: &mut D,
        dec2: &mut D,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        ssim::calculate_video_ssim(dec1, dec2, None)
    }

    fn print_results(result: Self::VideoResult) {
        println!(
            "     {:<10} →  Y: {:<8.4} U/Cb: {:<8.4} V/Cr: {:<8.4} Avg value: {:<8.4}",
            style("SSIM").cyan(),
            result.y,
            result.u,
            result.v,
            result.avg
        );
    }
}

struct MsSsim;

impl CliMetric for MsSsim {
    type VideoResult = PlanarMetrics;

    fn calculate_video_metric<D: Decoder>(
        dec1: &mut D,
        dec2: &mut D,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        ssim::calculate_video_msssim(dec1, dec2, None)
    }

    fn print_results(result: Self::VideoResult) {
        println!(
            "     {:<10} →  Y: {:<8.4} U/Cb: {:<8.4} V/Cr: {:<8.4} Avg value: {:<8.4}",
            style("MSSSIM").cyan(),
            result.y,
            result.u,
            result.v,
            result.avg
        );
    }
}

struct Ciede2000;

impl CliMetric for Ciede2000 {
    type VideoResult = f64;

    fn calculate_video_metric<D: Decoder>(
        dec1: &mut D,
        dec2: &mut D,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        ciede::calculate_video_ciede(dec1, dec2, None)
    }

    fn print_results(result: Self::VideoResult) {
        println!(
            "     {:<10} →  Delta: {:<8.4}",
            style("CIEDE2000").cyan(),
            result
        )
    }
}
