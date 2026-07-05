use anyhow::{ensure, Result};
use av_metrics::video::{
    decode::{Decoder, Rational, VideoDetails},
    ChromaSubsampling,
};
use std::{
    mem::{size_of, transmute},
    path::Path,
};
use vapoursynth::{
    format::Format,
    prelude::*,
    video_info::{Framerate, Resolution},
};

/// A video decoder implementation using Vapoursynth
pub struct VapoursynthDecoder {
    env: Environment,
    cur_frame: usize,
}

impl VapoursynthDecoder {
    /// Loads a video file using the default decoder plugin (currently `LSmashSource`)
    pub fn new_from_video(filename: &Path) -> Result<Self> {
        Self::new_from_video_with_decoder(filename, VapoursynthDecoderPlugin::default())
    }

    /// Loads a video file using the specified decoder plugin
    pub fn new_from_video_with_decoder(
        filename: &Path,
        plugin: VapoursynthDecoderPlugin,
    ) -> Result<Self> {
        let escaped_filename = filename
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        let script = format!(
            r#"
import vapoursynth as vs

core = vs.core

clip = {}(source="{}"{})
clip.set_output(0)
        "#,
            match plugin {
                VapoursynthDecoderPlugin::LSmash => "core.lsmas.LWLibavSource",
                VapoursynthDecoderPlugin::BestSource => "core.bs.VideoSource",
            },
            escaped_filename,
            if plugin == VapoursynthDecoderPlugin::BestSource {
                ", cachepath=\"/\""
            } else {
                ""
            }
        );
        let env = Environment::from_script(&script)?;
        let this = Self { env, cur_frame: 0 };
        this.get_node()?;
        ensure!(
            this.get_format()?.sample_type() == SampleType::Integer,
            "Currently only integer input is supported"
        );
        Ok(this)
    }

    /// Loads a `.vpy` script
    pub fn new_from_script(filename: &Path) -> Result<Self> {
        let env = Environment::from_file(filename, EvalFlags::SetWorkingDir)?;
        let this = Self { env, cur_frame: 0 };
        this.get_node()?;
        ensure!(
            this.get_format()?.sample_type() == SampleType::Integer,
            "Currently only integer input is supported"
        );
        Ok(this)
    }

    fn get_node(&self) -> Result<Node<'_>> {
        Ok(self.env.get_output(0)?.0)
    }

    fn get_resolution(&self) -> Result<Resolution> {
        match self.get_node()?.info().resolution {
            Property::Constant(res) => Ok(res),
            Property::Variable => Err(anyhow::anyhow!(
                "Variable resolution videos are not supported"
            )),
        }
    }

    fn get_format(&self) -> Result<Format<'_>> {
        Ok(self.get_node()?.info().format)
    }

    fn get_frame_rate(&self) -> Result<Framerate> {
        match self.get_node()?.info().framerate {
            Property::Constant(fps) => Ok(fps),
            Property::Variable => Err(anyhow::anyhow!(
                "Variable frameratevideos are not supported"
            )),
        }
    }

    /// Returns the number of frames in this video
    pub fn get_frame_count(&self) -> Result<usize> {
        Ok(self.get_node()?.info().num_frames)
    }
}

impl Decoder for VapoursynthDecoder {
    fn read_video_frame<T: av_metrics::video::Pixel>(
        &mut self,
    ) -> Option<av_metrics::video::Frame<T>> {
        let details = self.get_video_details();
        if details.bit_depth <= 8 {
            assert!(size_of::<T>() == 1);
        } else if details.bit_depth <= 16 {
            assert!(size_of::<T>() == 2);
        } else {
            panic!("Unsupported bit depth");
        }

        let mut f: av_metrics::video::Frame<T> = av_metrics::video::FrameBuilder::new(
            details.width,
            details.height,
            details.chroma_sampling,
            details.bit_depth as u8,
        )
        .build()
        .expect("can build frame");

        {
            let Ok(frame) = self.get_node().unwrap().get_frame(self.cur_frame) else {
                return None;
            };

            match size_of::<T>() {
                1 => {
                    for (out_row, in_row) in f
                        .y_plane
                        .rows_mut()
                        .zip((0..details.height).map(|y| frame.plane_row::<u8>(0, y)))
                    {
                        // SAFETY: We know that `T` is `u8` here.
                        out_row[..in_row.len()].copy_from_slice(unsafe { transmute(in_row) });
                    }
                    if let Some((_, y_ratio)) = details.chroma_sampling.subsample_ratio() {
                        for (out_row, in_row) in
                            f.plane_mut(1).expect("has plane 1").rows_mut().zip(
                                (0..(details.height / usize::from(y_ratio.get())))
                                    .map(|y| frame.plane_row::<u8>(1, y)),
                            )
                        {
                            // SAFETY: We know that `T` is `u8` here.
                            out_row[..in_row.len()].copy_from_slice(unsafe { transmute(in_row) });
                        }

                        for (out_row, in_row) in
                            f.plane_mut(2).expect("has plane 2").rows_mut().zip(
                                (0..(details.height / usize::from(y_ratio.get())))
                                    .map(|y| frame.plane_row::<u8>(2, y)),
                            )
                        {
                            // SAFETY: We know that `T` is `u8` here.
                            out_row[..in_row.len()].copy_from_slice(unsafe { transmute(in_row) });
                        }
                    }
                }
                2 => {
                    for (out_row, in_row) in f
                        .y_plane
                        .rows_mut()
                        .zip((0..details.height).map(|y| frame.plane_row::<u16>(0, y)))
                    {
                        // SAFETY: We know that `T` is `u16` here.
                        out_row[..in_row.len()].copy_from_slice(unsafe { transmute(in_row) });
                    }
                    if let Some((_, y_ratio)) = details.chroma_sampling.subsample_ratio() {
                        for (out_row, in_row) in
                            f.plane_mut(1).expect("has plane 1").rows_mut().zip(
                                (0..(details.height / usize::from(y_ratio.get())))
                                    .map(|y| frame.plane_row::<u16>(1, y)),
                            )
                        {
                            // SAFETY: We know that `T` is `u16` here.
                            out_row[..in_row.len()].copy_from_slice(unsafe { transmute(in_row) });
                        }

                        for (out_row, in_row) in
                            f.plane_mut(2).expect("has plane 2").rows_mut().zip(
                                (0..(details.height / usize::from(y_ratio.get())))
                                    .map(|y| frame.plane_row::<u16>(2, y)),
                            )
                        {
                            // SAFETY: We know that `T` is `u16` here.
                            out_row[..in_row.len()].copy_from_slice(unsafe { transmute(in_row) });
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        self.cur_frame += 1;
        Some(f)
    }

    fn get_bit_depth(&self) -> usize {
        let format = self.get_format().unwrap();
        format.bits_per_sample() as usize
    }

    fn get_video_details(&self) -> VideoDetails {
        let format = self.get_format().unwrap();
        let res = self.get_resolution().unwrap();
        let fps = self.get_frame_rate().unwrap();
        let chroma = match (
            format.color_family(),
            format.sub_sampling_w() + format.sub_sampling_h(),
        ) {
            (ColorFamily::Gray, _) => ChromaSubsampling::Monochrome,
            (_, 0) => ChromaSubsampling::Yuv444,
            (_, 1) => ChromaSubsampling::Yuv422,
            _ => ChromaSubsampling::Yuv420,
        };
        VideoDetails {
            width: res.width,
            height: res.height,
            bit_depth: format.bits_per_sample() as usize,
            chroma_sampling: chroma,
            chroma_sample_position: av_metrics::video::ChromaSamplePosition::Unknown,
            time_base: Rational::new(fps.denominator, fps.numerator),
            luma_padding: 0,
        }
    }
}

/// Specifies a Vapoursynth plugin to use for video decoding.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum VapoursynthDecoderPlugin {
    /// L-SMASH decoding (default), custom video demuxer and decoder
    #[default]
    LSmash,
    /// BestSource, cross-platform wrapper around FFmpeg
    BestSource,
}
