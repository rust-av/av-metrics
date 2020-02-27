use crate::video::decode::Decoder;
use crate::video::decode::Rational;
use crate::video::decode::VideoDetails;
use crate::video::pixel::Pixel;
use crate::video::{ChromaSamplePosition, ChromaSampling, FrameInfo};
use std::cmp;
use std::io::Read;
use v_frame::frame::Frame;
use v_frame::pixel::CastFromPrimitive;
use v_frame::plane::Plane;

/// Function to map y4m color space
pub fn map_y4m_color_space(color_space: y4m::Colorspace) -> (ChromaSampling, ChromaSamplePosition) {
    use crate::video::ChromaSamplePosition::*;
    use crate::video::ChromaSampling::*;
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

impl<R: Read> Decoder for y4m::Decoder<'_, R> {
    fn get_video_details(&self) -> VideoDetails {
        let width = self.get_width();
        let height = self.get_height();
        let color_space = self.get_colorspace();
        let bit_depth = color_space.get_bit_depth();
        let (chroma_sampling, chroma_sample_position) = map_y4m_color_space(color_space);
        let framerate = self.get_framerate();
        let time_base = Rational::new(framerate.den as u64, framerate.num as u64);
        let luma_padding = 0;

        VideoDetails {
            width,
            height,
            bit_depth,
            chroma_sampling,
            chroma_sample_position,
            time_base,
            luma_padding,
        }
    }

    fn read_video_frame<T: Pixel>(&mut self, cfg: &VideoDetails) -> Result<FrameInfo<T>, ()> {
        let bit_depth = self.get_bit_depth();
        let color_space = self.get_colorspace();
        let (chroma_sampling, chroma_sample_pos) = map_y4m_color_space(color_space);
        let bytes = self.get_bytes_per_sample();
        self.read_frame()
            .map(|frame| {
                let mut f: Frame<T> = Frame::new_with_padding(
                    cfg.width,
                    cfg.height,
                    cfg.chroma_sampling,
                    cfg.luma_padding,
                );

                let (chroma_width, _) = cfg
                    .chroma_sampling
                    .get_chroma_dimensions(cfg.width, cfg.height);
                f.planes[0].copy_from_raw_u8(frame.get_y_plane(), cfg.width * bytes, bytes);
                f.planes[1].copy_from_raw_u8(frame.get_u_plane(), chroma_width * bytes, bytes);
                f.planes[2].copy_from_raw_u8(frame.get_v_plane(), chroma_width * bytes, bytes);
                f.planes[1] = convert_chroma_data(&f.planes[1], chroma_sample_pos, bit_depth);
                f.planes[2] = convert_chroma_data(&f.planes[2], chroma_sample_pos, bit_depth);

                FrameInfo {
                    bit_depth,
                    chroma_sampling,
                    planes: f.planes,
                }
            })
            .map_err(|_| ())
    }

    fn read_specific_frame<T: Pixel>(&mut self, frame_number: usize) -> Result<FrameInfo<T>, ()> {
        let mut frame_no = 0;
        while frame_no <= frame_number {
            let frame = self.read_video_frame(&self.get_video_details());
            if frame_no == frame_number {
                if let Ok(frame) = frame {
                    return Ok(frame);
                }
            }
            frame_no += 1;
        }
        Err(())
    }

    fn get_bit_depth(&self) -> usize {
        self.get_bit_depth()
    }
}

/// The algorithms (as ported from daala-tools) expect a colocated or bilaterally located chroma
/// sample position. This means that a vertical chroma sample position must be realigned
/// in order to produce a correct result.
///
/// TODO: Take y4m frame as input data with chroma_width and bytes as parameters and returns
/// newly constructed frame. The function prototype could be like
/// convert_chroma_data(frame.get_u_plane(), chroma_sample_pos, bit_depth, chroma_width, bytes);
fn convert_chroma_data<T: Pixel>(
    plane_data: &Plane<T>,
    chroma_pos: ChromaSamplePosition,
    bit_depth: usize,
) -> Plane<T> {
    if chroma_pos != ChromaSamplePosition::Vertical {
        // TODO: Also convert Interpolated chromas
        return plane_data.clone();
    }
    let mut output_data = plane_data.data.clone();
    let width = plane_data.cfg.width;
    let height = plane_data.cfg.height;
    for y in 0..height {
        // Filter: [4 -17 114 35 -9 1]/128, derived from a 6-tap Lanczos window.
        let in_row = &plane_data.data[(y * width)..];
        let out_row = &mut output_data[(y * width)..];
        let breakpoint = cmp::min(width, 2);
        for x in 0..breakpoint {
            out_row[x] = T::cast_from(clamp(
                (4 * i32::cast_from(in_row[0]) - 17 * i32::cast_from(in_row[x.saturating_sub(1)])
                    + 114 * i32::cast_from(in_row[x])
                    + 35 * i32::cast_from(in_row[cmp::min(x + 1, width - 1)])
                    - 9 * i32::cast_from(in_row[cmp::min(x + 2, width - 1)])
                    + i32::cast_from(in_row[cmp::min(x + 3, width - 1)])
                    + 64)
                    >> 7,
                0,
                (1 << bit_depth) - 1,
            ));
        }
        let breakpoint2 = width - 3;
        for x in breakpoint..breakpoint2 {
            out_row[x] = T::cast_from(clamp(
                (4 * i32::cast_from(in_row[x - 2]) - 17 * i32::cast_from(in_row[x - 1])
                    + 114 * i32::cast_from(in_row[x])
                    + 35 * i32::cast_from(in_row[x + 1])
                    - 9 * i32::cast_from(in_row[x + 2])
                    + i32::cast_from(in_row[x + 3])
                    + 64)
                    >> 7,
                0,
                (1 << bit_depth) - 1,
            ));
        }
        for x in breakpoint2..width {
            out_row[x] = T::cast_from(clamp(
                (4 * i32::cast_from(in_row[x - 2]) - 17 * i32::cast_from(in_row[x - 1])
                    + 114 * i32::cast_from(in_row[x])
                    + 35 * i32::cast_from(in_row[cmp::min(x + 1, width - 1)])
                    - 9 * i32::cast_from(in_row[cmp::min(x + 2, width - 1)])
                    + i32::cast_from(in_row[width - 1])
                    + 64)
                    >> 7,
                0,
                (1 << bit_depth) - 1,
            ));
        }
    }
    Plane {
        data: output_data,
        cfg: plane_data.cfg.clone(),
    }
}

#[inline]
fn clamp<T: PartialOrd>(input: T, min: T, max: T) -> T {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}
