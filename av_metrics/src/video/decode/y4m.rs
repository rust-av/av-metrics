use crate::video::decode::Decoder;
use crate::video::decode::Rational;
use crate::video::decode::VideoDetails;
use crate::video::pixel::Pixel;
use crate::video::{convert_chroma_data, ChromaSamplePosition, ChromaSampling, FrameInfo};
use std::io::Read;
use v_frame::frame::Frame;

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

impl<R: Read + Send + Sync> Decoder for y4m::Decoder<R> {
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

    fn read_video_frame<T: Pixel>(&mut self) -> Option<FrameInfo<T>> {
        let bit_depth = self.get_bit_depth();
        let color_space = self.get_colorspace();
        let (chroma_sampling, chroma_sample_pos) = map_y4m_color_space(color_space);
        let width = self.get_width();
        let height = self.get_height();
        let bytes = self.get_bytes_per_sample();
        self.read_frame().ok().map(|frame| {
            let mut f: Frame<T> = Frame::new_with_padding(width, height, chroma_sampling, 0);

            let (chroma_width, _) = chroma_sampling.get_chroma_dimensions(width, height);
            f.planes[0].copy_from_raw_u8(frame.get_y_plane(), width * bytes, bytes);
            convert_chroma_data(
                &mut f.planes[1],
                chroma_sample_pos,
                bit_depth,
                frame.get_u_plane(),
                chroma_width * bytes,
                bytes,
            );
            convert_chroma_data(
                &mut f.planes[2],
                chroma_sample_pos,
                bit_depth,
                frame.get_v_plane(),
                chroma_width * bytes,
                bytes,
            );

            FrameInfo {
                bit_depth,
                chroma_sampling,
                planes: f.planes,
            }
        })
    }

    fn get_bit_depth(&self) -> usize {
        self.get_bit_depth()
    }
}
