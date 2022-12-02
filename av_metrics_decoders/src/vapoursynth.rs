use anyhow::{ensure, Result};
use av_metrics::video::{
    decode::{Decoder, Rational, VideoDetails},
    ChromaSampling,
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

/// A video decoder implementation using Vaopursynth
pub struct VapoursynthDecoder {
    env: Environment,
    cur_frame: usize,
}

impl VapoursynthDecoder {
    /// Loads a video file using `LSmashSource`
    pub fn new_from_video(filename: &Path) -> Result<Self> {
        let script = format!(
            r#"
import vapoursynth as vs

core = vs.core

clip = core.lsmas.LWLibavSource(source="{}")
clip.set_output(0)
        "#,
            filename
                .canonicalize()
                .unwrap()
                .to_string_lossy()
                .trim_start_matches(r"\\?\")
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
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
        match self.get_node()?.info().format {
            Property::Constant(format) => Ok(format),
            Property::Variable => Err(anyhow::anyhow!("Variable format videos are not supported")),
        }
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
        assert!(details.bit_depth == size_of::<T>());

        let mut f: av_metrics::video::Frame<T> = av_metrics::video::Frame::new_with_padding(
            details.width,
            details.height,
            details.chroma_sampling,
            0,
        );

        {
            let frame = self.get_node().unwrap().get_frame(self.cur_frame);
            if frame.is_err() {
                return None;
            }
            let frame = frame.unwrap();
            match size_of::<T>() {
                1 => {
                    for (out_row, in_row) in f.planes[0]
                        .rows_iter_mut()
                        .zip((0..details.height).map(|y| frame.plane_row::<u8>(0, y)))
                    {
                        // SAFETY: We know that `T` is `u8` here.
                        out_row[..in_row.len()].copy_from_slice(unsafe { transmute(in_row) });
                    }
                    if details.chroma_sampling != ChromaSampling::Cs400 {
                        for (out_row, in_row) in f.planes[1].rows_iter_mut().zip(
                            (0..(details.height
                                >> details.chroma_sampling.get_decimation().unwrap().1))
                                .map(|y| frame.plane_row::<u8>(1, y)),
                        ) {
                            // SAFETY: We know that `T` is `u8` here.
                            out_row[..in_row.len()].copy_from_slice(unsafe { transmute(in_row) });
                        }
                    }
                    if details.chroma_sampling != ChromaSampling::Cs400 {
                        for (out_row, in_row) in f.planes[2].rows_iter_mut().zip(
                            (0..(details.height
                                >> details.chroma_sampling.get_decimation().unwrap().1))
                                .map(|y| frame.plane_row::<u8>(2, y)),
                        ) {
                            // SAFETY: We know that `T` is `u8` here.
                            out_row[..in_row.len()].copy_from_slice(unsafe { transmute(in_row) });
                        }
                    }
                }
                2 => {
                    for (out_row, in_row) in f.planes[0]
                        .rows_iter_mut()
                        .zip((0..details.height).map(|y| frame.plane_row::<u16>(0, y)))
                    {
                        // SAFETY: We know that `T` is `u16` here.
                        out_row[..in_row.len()].copy_from_slice(unsafe { transmute(in_row) });
                    }
                    if details.chroma_sampling != ChromaSampling::Cs400 {
                        for (out_row, in_row) in f.planes[1].rows_iter_mut().zip(
                            (0..(details.height
                                >> details.chroma_sampling.get_decimation().unwrap().1))
                                .map(|y| frame.plane_row::<u16>(1, y)),
                        ) {
                            // SAFETY: We know that `T` is `u16` here.
                            out_row[..in_row.len()].copy_from_slice(unsafe { transmute(in_row) });
                        }
                    }
                    if details.chroma_sampling != ChromaSampling::Cs400 {
                        for (out_row, in_row) in f.planes[2].rows_iter_mut().zip(
                            (0..(details.height
                                >> details.chroma_sampling.get_decimation().unwrap().1))
                                .map(|y| frame.plane_row::<u16>(2, y)),
                        ) {
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
            (ColorFamily::Gray, _) => ChromaSampling::Cs400,
            (_, 0) => ChromaSampling::Cs444,
            (_, 1) => ChromaSampling::Cs422,
            _ => ChromaSampling::Cs420,
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
