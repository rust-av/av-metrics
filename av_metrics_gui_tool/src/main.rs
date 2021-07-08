#![windows_subsystem = "windows"]

mod input_output;
mod metrics;

// TODO
// 1. Replace unwrap() with unwrap_unchecked() when it hits stable
//    (no panic because the indexmap key is an enum)
// 2. Replace buttons with toggles as soon as iced hits 0.4

use iced::{
    button, executor, scrollable, Align, Application, Button, Clipboard, Column, Command,
    Container, Element, Length, Row, Scrollable, Settings, Space, Text,
};

use indexmap::{indexmap, IndexMap};

use crate::input_output::{get_root_path, FileType, SaveError, SavedState};

use crate::metrics::*;

const COLUMN_SPACING: u16 = 10;
const ROW_SPACING: u16 = 10;
const PADDING: u16 = 5;
const SPACE_UNITS: u16 = 30;
const ERROR_TEXT_COLOR: [f32; 3] = [1., 0., 0.];
const FILE_TEXT_COLOR: [f32; 3] = [0., 1., 0.];
const METRIC_TEXT_COLOR: [f32; 3] = [0., 0., 1.];
const FILETYPE: FileType = if cfg!(feature = "ffmpeg") || cfg!(feature = "ffmpeg_static") {
    FileType::FFmpeg
} else {
    FileType::Y4m
};

pub fn main() -> iced::Result {
    AvMetricsGui::run(Settings {
        ..Settings::default()
    })
}

#[derive(Default)]
struct AvMetricsGui {
    load_first_video: button::State,
    load_second_video: button::State,
    psnr: button::State,
    apsnr: button::State,
    psnr_hvs: button::State,
    ssim: button::State,
    msssim: button::State,
    ciede2000: button::State,
    all: button::State,
    export: button::State,
    scroll: scrollable::State,
    is_first_loaded: bool,
    is_second_loaded: bool,
    is_saving: bool,
    path1: String,
    path2: String,
    error: String,
    planar_metrics: IndexMap<PlanarMetric, MetricData<PlanarType>>,
    ciede_metric: MetricData<f64>,
}

impl AvMetricsGui {
    fn new() -> Self {
        let planar_metrics = indexmap! {
                   PlanarMetric::Psnr => MetricData::<PlanarType>::new("PSNR"),
                   PlanarMetric::APsnr => MetricData::<PlanarType>::new("APSNR"),
                   PlanarMetric::PsnrHvs => MetricData::<PlanarType>::new("PSNR_HVS"),
                   PlanarMetric::Ssim => MetricData::<PlanarType>::new("SSIM"),
                   PlanarMetric::MsSsim => MetricData::<PlanarType>::new("MSSSIM"),
        };

        let ciede_metric = MetricData::<f64>::new("Ciede2000");

        Self {
            planar_metrics,
            ciede_metric,
            ..Self::default()
        }
    }

    fn clear_state(&mut self) {
        if self.are_there_metrics() {
            self.planar_metrics.values_mut().for_each(|planar_metric| {
                planar_metric.reset();
            });

            self.ciede_metric.reset();
        }
    }

    fn compute_planar_metric(&mut self, metric_name: PlanarMetric) -> Command<Message> {
        let planar_metric = self.planar_metrics.get_mut(&metric_name).unwrap();

        if planar_metric.state.show {
            self.planar_metrics
                .iter_mut()
                .for_each(|(name, planar_metric)| planar_metric.state.show = *name == metric_name);
            self.ciede_metric.state.show = false;
        } else if planar_metric.state.is_computed {
            planar_metric.state.show = true;
        } else if !planar_metric.state.is_computing {
            planar_metric.state.is_computing = true;
            let path1 = self.path1.clone();
            let path2 = self.path2.clone();
            return match metric_name {
                PlanarMetric::Psnr => {
                    Command::perform(Psnr::run(path1, path2), Message::ComputedPlanarMetrics)
                }
                PlanarMetric::APsnr => {
                    Command::perform(APsnr::run(path1, path2), Message::ComputedPlanarMetrics)
                }
                PlanarMetric::PsnrHvs => {
                    Command::perform(PsnrHvs::run(path1, path2), Message::ComputedPlanarMetrics)
                }
                PlanarMetric::Ssim => {
                    Command::perform(Ssim::run(path1, path2), Message::ComputedPlanarMetrics)
                }
                PlanarMetric::MsSsim => {
                    Command::perform(MsSsim::run(path1, path2), Message::ComputedPlanarMetrics)
                }
            };
        }
        Command::none()
    }

    fn compute_ciede(&mut self) -> Command<Message> {
        if self.ciede_metric.state.show {
            self.planar_metrics
                .values_mut()
                .for_each(|planar_metric| planar_metric.state.show = false);
        } else if self.ciede_metric.state.is_computed {
            self.ciede_metric.state.show = true;
        } else if !self.ciede_metric.state.is_computing {
            self.ciede_metric.state.is_computing = true;
            return Command::perform(
                Ciede2000::run(self.path1.clone(), self.path2.clone()),
                Message::ComputedCiede,
            );
        }
        Command::none()
    }

    #[inline(always)]
    fn is_computing(&self) -> bool {
        self.planar_metrics
            .values()
            .any(|planar_metric| planar_metric.state.is_computing)
            || self.ciede_metric.state.is_computing
    }

    #[inline(always)]
    fn are_there_metrics(&self) -> bool {
        self.planar_metrics
            .values()
            .any(|planar_metric| planar_metric.state.is_computed)
            || self.ciede_metric.state.is_computed
    }

    fn save_file(&mut self, path: String) -> Command<Message> {
        let planar_values: Vec<Option<PlanarType>> = self
            .planar_metrics
            .values()
            .map(|planar_metric| {
                if planar_metric.state.show {
                    planar_metric.value
                } else {
                    None
                }
            })
            .collect();
        let ciede_value = if self.ciede_metric.state.show {
            self.ciede_metric.value
        } else {
            None
        };
        self.is_saving = true;
        Command::perform(
            SavedState {
                metrics: MetricsAggregator {
                    video1: self.path1.clone(),
                    video2: self.path2.clone(),
                    psnr: planar_values[0],
                    apsnr: planar_values[1],
                    psnr_hvs: planar_values[2],
                    ssim: planar_values[3],
                    msssim: planar_values[4],
                    ciede2000: ciede_value,
                },
                path,
            }
            .save(),
            Message::Saved,
        )
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadFirstVideoRequest,
    LoadSecondVideoRequest,
    LoadFirstVideo(Option<String>),
    LoadSecondVideo(Option<String>),
    SaveAs,
    SaveTo(Option<String>),
    Saved(Result<(), SaveError>),
    All,
    Psnr,
    APsnr,
    PsnrHvs,
    Ssim,
    MsSsim,
    Ciede2000,
    ComputedPlanarMetrics((PlanarMetric, Result<PlanarType, String>)),
    ComputedCiede(Result<f64, String>),
}

impl Application for AvMetricsGui {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Message>) {
        (AvMetricsGui::new(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Av Metrics Gui")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::LoadFirstVideoRequest => {
                if cfg!(target_os = "macos") {
                    if let Some(path1) = crate::input_output::select_macos_file(
                        false,
                        FILETYPE,
                        std::env::current_dir().ok(),
                    ) {
                        if self.path1 != path1 {
                            self.clear_state();
                            self.error.clear();
                            self.path1 = path1;
                            self.is_first_loaded = true;
                        }
                    }
                } else if !self.is_computing() {
                    return Command::perform(
                        crate::input_output::select_file(
                            false,
                            FILETYPE,
                            std::env::current_dir().ok(),
                        ),
                        Message::LoadFirstVideo,
                    );
                }
            }
            Message::LoadFirstVideo(Some(path1)) => {
                if self.path1 != path1 {
                    self.clear_state();
                    self.error.clear();
                    self.path1 = path1;
                    self.is_first_loaded = true;
                }
            }
            Message::LoadSecondVideoRequest => {
                if cfg!(target_os = "macos") {
                    if let Some(path2) = crate::input_output::select_macos_file(
                        false,
                        FILETYPE,
                        std::env::current_dir().ok(),
                    ) {
                        if self.path2 != path2 {
                            self.clear_state();
                            self.error.clear();
                            self.path2 = path2;
                            self.is_second_loaded = true;
                        }
                    }
                } else if !self.is_computing() {
                    return Command::perform(
                        crate::input_output::select_file(
                            false,
                            FILETYPE,
                            std::env::current_dir().ok(),
                        ),
                        Message::LoadSecondVideo,
                    );
                }
            }
            Message::LoadSecondVideo(Some(path2)) => {
                if self.path2 != path2 {
                    self.clear_state();
                    self.error.clear();
                    self.path2 = path2;
                    self.is_second_loaded = true;
                }
            }
            Message::SaveAs => {
                if cfg!(target_arch = "wasm32") {
                    if let Ok(path) = get_root_path()
                        .join("wasm_metric.json")
                        .into_os_string()
                        .into_string()
                    {
                        return self.save_file(path);
                    } else {
                        self.error = "Error getting the saving path".to_owned();
                    }
                } else if cfg!(target_os = "macos") {
                    if let Some(path) = crate::input_output::select_macos_file(
                        true,
                        FileType::Json,
                        std::env::current_dir().ok(),
                    ) {
                        return self.save_file(path);
                    }
                } else {
                    return Command::perform(
                        crate::input_output::select_file(
                            true,
                            FileType::Json,
                            std::env::current_dir().ok(),
                        ),
                        Message::SaveTo,
                    );
                }
            }
            Message::Saved(_) => {
                self.is_saving = false;
            }
            Message::SaveTo(Some(path)) => {
                if !self.is_saving {
                    return self.save_file(path);
                }
            }
            Message::LoadFirstVideo(None)
            | Message::LoadSecondVideo(None)
            | Message::SaveTo(None) => {}
            Message::ComputedCiede(metric_res) => {
                self.ciede_metric.state.is_computing = false;
                match metric_res {
                    Ok(val) => self.ciede_metric.update(val),
                    Err(e) => self.error = e,
                }
            }
            Message::ComputedPlanarMetrics(metric_res) => {
                let planar_metric = self.planar_metrics.get_mut(&metric_res.0).unwrap();
                planar_metric.state.is_computing = false;

                match metric_res.1 {
                    Ok(val) => planar_metric.update(val),
                    Err(e) => self.error = e,
                }
            }
            Message::All => {
                self.planar_metrics
                    .values_mut()
                    .filter(|planar_metric| planar_metric.state.is_computed)
                    .for_each(|planar_metric| planar_metric.state.show = true);

                #[allow(clippy::needless_collect)] // FIXME in some way
                let planar_metric_names: Vec<PlanarMetric> = self
                    .planar_metrics
                    .iter()
                    .filter(|(_, planar_metric)| !planar_metric.state.is_computed)
                    .map(|(metric_name, _)| metric_name.clone())
                    .collect();

                let mut metric_launcher: Vec<Command<Message>> = planar_metric_names
                    .into_iter()
                    .map(|metric_name| self.compute_planar_metric(metric_name))
                    .collect();

                if self.ciede_metric.state.is_computed {
                    self.ciede_metric.state.show = true;
                } else {
                    metric_launcher.push(self.compute_ciede());
                }

                if !metric_launcher.is_empty() {
                    return Command::batch(metric_launcher.into_iter());
                }
            }
            Message::Psnr => return self.compute_planar_metric(PlanarMetric::Psnr),
            Message::APsnr => return self.compute_planar_metric(PlanarMetric::APsnr),
            Message::PsnrHvs => return self.compute_planar_metric(PlanarMetric::PsnrHvs),
            Message::Ssim => return self.compute_planar_metric(PlanarMetric::Ssim),
            Message::MsSsim => return self.compute_planar_metric(PlanarMetric::MsSsim),
            Message::Ciede2000 => return self.compute_ciede(),
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let is_not_computing = !self.is_computing();
        let are_there_metrics = self.are_there_metrics();

        let mut header_columns = vec![
            Text::new(&self.error).color(ERROR_TEXT_COLOR).into(),
            Row::new()
                .spacing(ROW_SPACING)
                .align_items(Align::Center)
                .push(
                    Button::new(&mut self.load_first_video, Text::new("Load first video"))
                        .on_press(Message::LoadFirstVideoRequest),
                )
                .push(
                    Button::new(&mut self.load_second_video, Text::new("Load second video"))
                        .on_press(Message::LoadSecondVideoRequest),
                )
                .into(),
        ];

        let mut file_row = Vec::new();
        if self.is_first_loaded {
            file_row.push(Text::new(&self.path1).color(FILE_TEXT_COLOR).into());
        }

        if self.is_second_loaded {
            file_row.push(Text::new(&self.path2).color(FILE_TEXT_COLOR).into());
        }

        if self.is_first_loaded || self.is_second_loaded {
            header_columns.push(
                Row::with_children(file_row)
                    .spacing(ROW_SPACING)
                    .align_items(Align::Center)
                    .into(),
            );
        }

        let header_column = Column::with_children(header_columns)
            .spacing(COLUMN_SPACING)
            .align_items(Align::Center);

        if !(self.error.is_empty() && self.is_first_loaded && self.is_second_loaded) {
            return Container::new(header_column)
                .padding(PADDING)
                .height(Length::Fill)
                .width(Length::Fill)
                .center_x()
                .into();
        }

        let metric_names: Vec<&'static str> = self
            .planar_metrics
            .values()
            .map(|planar_metric| planar_metric.name)
            .collect();

        let mut row_buttons = vec![
            create_button(&mut self.psnr, metric_names[0], Message::Psnr),
            create_button(&mut self.apsnr, metric_names[1], Message::APsnr),
            create_button(&mut self.psnr_hvs, metric_names[2], Message::PsnrHvs),
            create_button(&mut self.ssim, metric_names[3], Message::Ssim),
            create_button(&mut self.msssim, metric_names[4], Message::MsSsim),
        ];

        row_buttons.push(create_button(
            &mut self.ciede2000,
            self.ciede_metric.name,
            Message::Ciede2000,
        ));

        row_buttons.push(create_button(&mut self.all, "All metrics", Message::All));

        if are_there_metrics && is_not_computing {
            row_buttons.push(create_button(
                &mut self.export,
                "Export metrics",
                Message::SaveAs,
            ));
        }

        let metrics_buttons = Column::new()
            .spacing(COLUMN_SPACING)
            .align_items(Align::Center)
            .push(header_column)
            .push(
                Row::with_children(row_buttons)
                    .spacing(ROW_SPACING)
                    .align_items(Align::Center),
            )
            .push(Space::new(Length::Fill, Length::Units(SPACE_UNITS)))
            .into();

        let mut metrics = vec![metrics_buttons];
        for planar_metric in self.planar_metrics.values() {
            if planar_metric.state.show {
                if let Some(metric_value) =
                    Gui::render_metric(planar_metric.name, planar_metric.value)
                {
                    metrics.push(metric_value);
                }
            }
        }

        if self.ciede_metric.state.show {
            if let Some(metric_value) =
                Gui::render_metric(self.ciede_metric.name, self.ciede_metric.value)
            {
                metrics.push(metric_value);
            }
        }

        let scroll = Scrollable::new(&mut self.scroll)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(
                Column::with_children(metrics)
                    .spacing(COLUMN_SPACING)
                    .align_items(Align::Center),
            );

        Container::new(scroll)
            .padding(PADDING)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

fn create_button<'a>(
    button_state: &'a mut button::State,
    name: &str,
    button_message: Message,
) -> Element<'a, Message> {
    Button::new(button_state, Text::new(name))
        .on_press(button_message)
        .into()
}

trait RenderMetric<T> {
    fn render_metric<'a>(
        metric_name: &str,
        metric_value: Option<T>,
    ) -> Option<Element<'a, Message>>;
}

struct Gui;

impl RenderMetric<PlanarType> for Gui {
    fn render_metric<'a>(
        metric_name: &str,
        metric_value: Option<PlanarType>,
    ) -> Option<Element<'a, Message>> {
        metric_value.map(|metric_value| {
            Column::new()
                .spacing(COLUMN_SPACING)
                .align_items(Align::Center)
                .push(Text::new(metric_name).color(METRIC_TEXT_COLOR))
                .push(
                    Row::new()
                        .spacing(ROW_SPACING)
                        .align_items(Align::Center)
                        .push(Text::new(format!("y: {}", metric_value.y)))
                        .push(Text::new(format!("u: {}", metric_value.u)))
                        .push(Text::new(format!("v: {}", metric_value.v)))
                        .push(Text::new(format!("avg: {}", metric_value.avg))),
                )
                .push(Space::new(Length::Fill, Length::Units(SPACE_UNITS)))
                .into()
        })
    }
}

impl RenderMetric<f64> for Gui {
    fn render_metric<'a>(
        metric_name: &str,
        metric_value: Option<f64>,
    ) -> Option<Element<'a, Message>> {
        metric_value.map(|metric_value| {
            Column::new()
                .spacing(COLUMN_SPACING)
                .align_items(Align::Center)
                .push(Text::new(metric_name).color(METRIC_TEXT_COLOR))
                .push(Text::new(format!("value: {}", metric_value)))
                .push(Space::new(Length::Fill, Length::Units(SPACE_UNITS)))
                .into()
        })
    }
}
