use av_metrics::video::*;
use clap::{App, Arg};
use console::style;
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Stdout, Write};
use std::path::Path;

fn main() -> Result<(), String> {
    let cli = App::new("AV Metrics")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("BASE")
                .help("The base input file to compare--currently supports Y4M files")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("FILES")
                .help("The alternate input files to compare with the base file")
                .required(true)
                .multiple(true)
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
                .long("export-json")
                .takes_value(true)
                .value_name("FILE"),
        )
        .arg(
            Arg::with_name("CSV")
                .help("Output results as CSV")
                .long("export-csv")
                .takes_value(true)
                .value_name("FILE"),
        )
        .arg(
            Arg::with_name("MARKDOWN")
                .help("Output results as Markdown")
                .long("export-markdown")
                .takes_value(true)
                .value_name("FILE"),
        )
        .arg(
            Arg::with_name("FILE")
                .help("Output results to a file")
                .long("export-file")
                .takes_value(true)
                .value_name("FILE"),
        )
        .arg(
            Arg::with_name("QUIET")
                .help("Do not output to stdout")
                .long("quiet")
                .takes_value(false),
        )
        .get_matches();
    let base = cli.value_of("BASE").unwrap();
    let inputs = cli.values_of("FILES").unwrap();
    let mut writers = vec![];
    if let Some(filename) = cli.value_of("FILE") {
        writers.push(OutputType::TEXT(BufWriter::new(
            File::create(filename).map_err(|err| err.to_string())?,
        )));
    };
    if let Some(filename) = cli.value_of("JSON") {
        writers.push(OutputType::JSON(BufWriter::new(
            File::create(filename).map_err(|err| err.to_string())?,
        )));
    };
    if let Some(filename) = cli.value_of("CSV") {
        writers.push(OutputType::CSV(BufWriter::new(
            File::create(filename).map_err(|err| err.to_string())?,
        )));
    };
    if let Some(filename) = cli.value_of("MARKDOWN") {
        writers.push(OutputType::Markdown(BufWriter::new(
            File::create(filename).map_err(|err| err.to_string())?,
        )));
    };
    if !cli.is_present("QUIET") {
        writers.push(OutputType::Stdout(BufWriter::new(std::io::stdout())));
    }

    let base_type = InputType::detect(base);

    let metrics = cli.value_of("METRIC");

    let mut report = Report::default();

    report.base = base;

    for input in inputs {
        let input_type = InputType::detect(input);

        match (base_type, input_type) {
            (InputType::Video, InputType::Video) => {
                report
                    .comparisons
                    .push(run_video_metrics(base, input, metrics));
            }
            (InputType::Audio, InputType::Audio) => {
                return Err("No audio metrics currently implemented, exiting.".to_owned());
            }
            (InputType::Video, InputType::Audio) | (InputType::Audio, InputType::Video) => {
                return Err("Incompatible input files.".to_owned());
            }
            (InputType::Unknown, _) | (_, InputType::Unknown) => {
                return Err("Unsupported input format.".to_owned());
            }
        };
    }

    for writer in writers.iter_mut() {
        report.print(writer)?;
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum InputType {
    Video,
    Audio,
    Unknown,
}

impl InputType {
    pub fn detect<P: AsRef<Path>>(_filename: P) -> Self {
        // FIXME: For now, just assume anything is a video, since that's all we currently support.
        InputType::Video
    }
}

#[cfg(not(feature = "ffmpeg"))]
pub fn get_decoder<P: AsRef<Path>>(input: P) -> Result<y4m::Decoder<File>, String> {
    let file = File::open(input).map_err(|e| e.to_string())?;
    y4m::Decoder::new(file).map_err(|e| e.to_string())
}

#[cfg(feature = "ffmpeg")]
pub fn get_decoder<P: AsRef<Path>>(input: P) -> Result<FfmpegDecoder, String> {
    FfmpegDecoder::new(input)
}

#[derive(Debug, Clone, Serialize, Default)]
struct MetricsResults {
    filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    psnr: Option<PlanarMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    apsnr: Option<PlanarMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    psnr_hvs: Option<PlanarMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ssim: Option<PlanarMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    msssim: Option<PlanarMetrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ciede2000: Option<f64>,
}

fn run_video_metrics(input1: &str, input2: &str, metric: Option<&str>) -> MetricsResults {
    let mut results = MetricsResults::default();

    results.filename = input2.to_owned();

    if metric.is_none() || metric == Some("psnr") {
        results.psnr = Psnr::run(input1, input2);
    }

    if metric.is_none() || metric == Some("apsnr") {
        results.apsnr = APsnr::run(input1, input2);
    }

    if metric.is_none() || metric == Some("psnrhvs") {
        results.psnr_hvs = PsnrHvs::run(input1, input2);
    }

    if metric.is_none() || metric == Some("ssim") {
        results.ssim = Ssim::run(input1, input2);
    }

    if metric.is_none() || metric == Some("msssim") {
        results.msssim = MsSsim::run(input1, input2);
    }

    if metric.is_none() || metric == Some("ciede2000") {
        results.ciede2000 = Ciede2000::run(input1, input2);
    }

    results
}

#[derive(Debug, Serialize, Default)]
struct Report<'s> {
    base: &'s str,
    comparisons: Vec<MetricsResults>,
}

impl Report<'_> {
    fn print(&self, writer: &mut OutputType) -> Result<(), String> {
        match writer {
            OutputType::JSON(w) => {
                writeln!(w, "{}", serde_json::to_string(&self).unwrap())
                    .map_err(|err| err.to_string())?;
            }
            OutputType::CSV(w) => {
                writeln!(w, "filename,psnr,apsnr,psnr_hvs,ssim,msssim,ciede2000")
                    .map_err(|err| err.to_string())?;
                for cmp in self.comparisons.iter() {
                    writeln!(
                        w,
                        "{},{},{},{},{},{},{}",
                        cmp.filename,
                        cmp.psnr.map(|v| v.avg).unwrap_or(-0.0),
                        cmp.apsnr.map(|v| v.avg).unwrap_or(-0.0),
                        cmp.psnr_hvs.map(|v| v.avg).unwrap_or(-0.0),
                        cmp.ssim.map(|v| v.avg).unwrap_or(-0.0),
                        cmp.msssim.map(|v| v.avg).unwrap_or(-0.0),
                        cmp.ciede2000.unwrap_or(-0.0)
                    )
                    .map_err(|err| err.to_string())?;
                }
            }
            OutputType::Markdown(w) => {
                writeln!(
                    w,
                    "|filename|psnr|apsnr|psnr_hvs|ssim|msssim|ciede2000|\n\
                     |-|-|-|-|-|-|-|"
                )
                .map_err(|err| err.to_string())?;
                for cmp in self.comparisons.iter() {
                    writeln!(
                        w,
                        "|{}|{}|{}|{}|{}|{}|{}|",
                        cmp.filename,
                        cmp.psnr.map(|v| v.avg).unwrap_or(-0.0),
                        cmp.apsnr.map(|v| v.avg).unwrap_or(-0.0),
                        cmp.psnr_hvs.map(|v| v.avg).unwrap_or(-0.0),
                        cmp.ssim.map(|v| v.avg).unwrap_or(-0.0),
                        cmp.msssim.map(|v| v.avg).unwrap_or(-0.0),
                        cmp.ciede2000.unwrap_or(-0.0)
                    )
                    .map_err(|err| err.to_string())?;
                }
            }
            OutputType::Stdout(_) | OutputType::TEXT(_) => {
                writeln!(writer, "Comparing {}\n", style(self.base).italic().cyan())
                    .map_err(|err| err.to_string())?;
                for cmp in self.comparisons.iter() {
                    writeln!(
                        writer,
                        "\n    {} for {}: \n",
                        style("Results").yellow(),
                        style(&cmp.filename).italic().cyan()
                    )
                    .map_err(|err| err.to_string())?;
                    Text::print_result(writer, "PSNR", cmp.psnr)?;
                    Text::print_result(writer, "APSNR", cmp.apsnr)?;
                    Text::print_result(writer, "PSNR HVS", cmp.psnr_hvs)?;
                    Text::print_result(writer, "SSIM", cmp.ssim)?;
                    Text::print_result(writer, "MSSSIM", cmp.msssim)?;
                    Text::print_result(writer, "CIEDE2000", cmp.ciede2000)?;
                }
            }
        }

        Ok(())
    }
}

enum OutputType {
    JSON(BufWriter<File>),
    CSV(BufWriter<File>),
    Markdown(BufWriter<File>),
    TEXT(BufWriter<File>),
    Stdout(BufWriter<Stdout>),
}

impl Write for OutputType {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            OutputType::JSON(f)
            | OutputType::CSV(f)
            | OutputType::Markdown(f)
            | OutputType::TEXT(f) => f.write(buf),
            OutputType::Stdout(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            OutputType::JSON(f)
            | OutputType::CSV(f)
            | OutputType::Markdown(f)
            | OutputType::TEXT(f) => f.flush(),
            OutputType::Stdout(s) => s.flush(),
        }
    }
}

trait CliMetric {
    type VideoResult: Serialize;

    fn run<P: AsRef<Path>>(input1: P, input2: P) -> Option<Self::VideoResult> {
        let mut dec1 = get_decoder(input1).expect("Failed to open input file 1");
        let mut dec2 = get_decoder(input2).expect("Failed to open input file 2");
        Self::calculate_video_metric(&mut dec1, &mut dec2).ok()
    }

    fn calculate_video_metric<D: Decoder>(
        dec1: &mut D,
        dec2: &mut D,
    ) -> Result<Self::VideoResult, Box<dyn Error>>;
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
}

trait PrintResult<T> {
    fn print_result(writer: &mut OutputType, header: &str, result: Option<T>)
        -> Result<(), String>;
}

struct Text;

impl PrintResult<PlanarMetrics> for Text {
    fn print_result(
        writer: &mut OutputType,
        header: &str,
        result: Option<PlanarMetrics>,
    ) -> Result<(), String> {
        if let Some(result) = result {
            writeln!(
                writer,
                "     {:<10} →  Y: {:<8.4} U/Cb: {:<8.4} V/Cr: {:<8.4} Avg value: {:<8.4}",
                style(header).cyan(),
                result.y,
                result.u,
                result.v,
                result.avg
            )
            .map_err(|err| err.to_string())?;
        }
        Ok(())
    }
}

impl PrintResult<f64> for Text {
    fn print_result(
        writer: &mut OutputType,
        header: &str,
        result: Option<f64>,
    ) -> Result<(), String> {
        if let Some(result) = result {
            writeln!(
                writer,
                "     {:<10} →  Delta: {:<8.4}",
                style(header).cyan(),
                result
            )
            .map_err(|err| err.to_string())?;
        }
        Ok(())
    }
}
