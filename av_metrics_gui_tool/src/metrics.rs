use std::error::Error;
use std::path::Path;

use async_trait::async_trait;
use serde::Serialize;

use av_metrics::video::decode::Decoder;
use av_metrics::video::PlanarMetrics;
use av_metrics::video::*;

#[cfg(any(feature = "ffmpeg", feature = "ffmpeg_static"))]
use av_metrics_decoders::FfmpegDecoder;
#[cfg(not(any(feature = "ffmpeg", feature = "ffmpeg_static")))]
use av_metrics_decoders::Y4MDecoder;

pub type PlanarType = PlanarMetrics;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlanarMetric {
    Psnr,
    APsnr,
    PsnrHvs,
    Ssim,
    MsSsim,
}

#[derive(Debug, Clone, Default)]
pub struct MetricState {
    pub is_computed: bool,
    pub is_computing: bool,
    pub show: bool,
}

impl MetricState {
    fn reset(&mut self) {
        self.is_computed = false;
        self.is_computing = false;
        self.show = false;
    }
}

pub trait MetricType {}

impl MetricType for PlanarType {}
impl MetricType for f64 {}

#[derive(Default)]
pub struct MetricData<T>
where
    T: MetricType + Default + Clone,
{
    pub name: &'static str,
    pub state: MetricState,
    pub value: Option<T>,
}

impl<T> MetricData<T>
where
    T: MetricType + Default + Clone,
{
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn update(&mut self, val: T) {
        self.state.is_computed = true;
        self.state.show = true;
        self.value = Some(val);
    }

    pub fn reset(&mut self) {
        self.state.reset();
        self.value = None;
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct MetricsAggregator {
    pub video1: String,
    pub video2: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psnr: Option<PlanarType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apsnr: Option<PlanarType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub psnr_hvs: Option<PlanarType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssim: Option<PlanarType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msssim: Option<PlanarType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ciede2000: Option<f64>,
}

#[cfg(not(any(feature = "ffmpeg", feature = "ffmpeg_static")))]
pub fn get_decoder<P: AsRef<Path>>(input: P) -> Result<Y4MDecoder, String> {
    Y4MDecoder::new(input)
}

#[cfg(any(feature = "ffmpeg", feature = "ffmpeg_static"))]
pub fn get_decoder<P: AsRef<Path>>(input: P) -> Result<FfmpegDecoder, String> {
    FfmpegDecoder::new(input)
}

#[async_trait]
pub trait PlanarMetricTrait {
    type VideoResult: Serialize;

    async fn run<P: AsRef<Path> + Send>(
        input1: P,
        input2: P,
    ) -> (PlanarMetric, Result<Self::VideoResult, String>) {
        let name = Self::name();

        let mut dec1 = match get_decoder(input1) {
            Ok(dec1) => dec1,
            Err(e) => return (name, Err(e)),
        };
        let mut dec2 = match get_decoder(input2) {
            Ok(dec2) => dec2,
            Err(e) => return (name, Err(e)),
        };
        (
            name,
            Self::calculate_video_metric(&mut dec1, &mut dec2, |_| ()).map_err(|e| e.to_string()),
        )
    }

    fn calculate_video_metric<D: Decoder, F: Fn(usize) + Send>(
        dec1: &mut D,
        dec2: &mut D,
        progress_callback: F,
    ) -> Result<Self::VideoResult, Box<dyn Error>>;

    fn name() -> PlanarMetric;
}

#[async_trait]
pub trait NonPlanarMetricTrait {
    type VideoResult: Serialize;

    async fn run<P: AsRef<Path> + Send>(input1: P, input2: P) -> Result<Self::VideoResult, String> {
        let mut dec1 = get_decoder(input1)?;
        let mut dec2 = get_decoder(input2)?;

        Self::calculate_video_metric(&mut dec1, &mut dec2, |_| ()).map_err(|e| e.to_string())
    }

    fn calculate_video_metric<D: Decoder, F: Fn(usize) + Send>(
        dec1: &mut D,
        dec2: &mut D,
        progress_callback: F,
    ) -> Result<Self::VideoResult, Box<dyn Error>>;
}

pub struct Psnr;

impl PlanarMetricTrait for Psnr {
    type VideoResult = PlanarType;

    fn calculate_video_metric<D: Decoder, F: Fn(usize) + Send>(
        dec1: &mut D,
        dec2: &mut D,
        progress_callback: F,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        psnr::calculate_video_psnr(dec1, dec2, None, progress_callback)
    }

    fn name() -> PlanarMetric {
        PlanarMetric::Psnr
    }
}

pub struct APsnr;

impl PlanarMetricTrait for APsnr {
    type VideoResult = PlanarType;

    fn calculate_video_metric<D: Decoder, F: Fn(usize) + Send>(
        dec1: &mut D,
        dec2: &mut D,
        progress_callback: F,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        psnr::calculate_video_apsnr(dec1, dec2, None, progress_callback)
    }

    fn name() -> PlanarMetric {
        PlanarMetric::APsnr
    }
}

pub struct PsnrHvs;

impl PlanarMetricTrait for PsnrHvs {
    type VideoResult = PlanarType;

    fn calculate_video_metric<D: Decoder, F: Fn(usize) + Send>(
        dec1: &mut D,
        dec2: &mut D,
        progress_callback: F,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        psnr_hvs::calculate_video_psnr_hvs(dec1, dec2, None, progress_callback)
    }

    fn name() -> PlanarMetric {
        PlanarMetric::PsnrHvs
    }
}

pub struct Ssim;

impl PlanarMetricTrait for Ssim {
    type VideoResult = PlanarType;

    fn calculate_video_metric<D: Decoder, F: Fn(usize) + Send>(
        dec1: &mut D,
        dec2: &mut D,
        progress_callback: F,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        ssim::calculate_video_ssim(dec1, dec2, None, progress_callback)
    }

    fn name() -> PlanarMetric {
        PlanarMetric::Ssim
    }
}

pub struct MsSsim;

impl PlanarMetricTrait for MsSsim {
    type VideoResult = PlanarType;

    fn calculate_video_metric<D: Decoder, F: Fn(usize) + Send>(
        dec1: &mut D,
        dec2: &mut D,
        progress_callback: F,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        ssim::calculate_video_msssim(dec1, dec2, None, progress_callback)
    }

    fn name() -> PlanarMetric {
        PlanarMetric::MsSsim
    }
}

pub struct Ciede2000;

impl NonPlanarMetricTrait for Ciede2000 {
    type VideoResult = f64;

    fn calculate_video_metric<D: Decoder, F: Fn(usize) + Send>(
        dec1: &mut D,
        dec2: &mut D,
        progress_callback: F,
    ) -> Result<Self::VideoResult, Box<dyn Error>> {
        ciede::calculate_video_ciede(dec1, dec2, None, progress_callback)
    }
}
