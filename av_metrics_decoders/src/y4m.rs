use av_metrics::video::decode::*;
use av_metrics::video::*;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// A decoder for a y4m input stream
pub struct Y4MDecoder {
    inner: y4m::Decoder<BufReader<File>>,
}

/// Function to map y4m color space
fn map_y4m_color_space(color_space: y4m::Colorspace) -> (ChromaSampling, ChromaSamplePosition) {
    use av_metrics::video::ChromaSamplePosition::*;
    use av_metrics::video::ChromaSampling::*;
    use y4m::Colorspace::*;
    match color_space {
        Cmono | Cmono12 => (Cs400, Unknown),
        C420jpeg => (Cs420, Bilateral),
        C420paldv => (Cs420, Interpolated),
        C420mpeg2 => (Cs420, Vertical),
        C420 | C420p10 | C420p12 => (Cs420, Colocated),
        C422 | C422p10 | C422p12 => (Cs422, Vertical),
        C444 | C444p10 | C444p12 => (Cs444, Colocated),
        _ => unimplemented!(),
    }
}

impl Y4MDecoder {
    /// Initialize a new Y4M decoder for a given input file
    pub fn new<P: AsRef<Path>>(input: P) -> Result<Self, String> {
        let file = File::open(input).map_err(|e| e.to_string())?;
        let inner = y4m::Decoder::new(BufReader::new(file)).map_err(|e| e.to_string())?;
        Ok(Self { inner })
    }
}

impl Decoder for Y4MDecoder {
    fn get_video_details(&self) -> VideoDetails {
        let width = self.inner.get_width();
        let height = self.inner.get_height();
        let color_space = self.inner.get_colorspace();
        let bit_depth = color_space.get_bit_depth();
        let (chroma_sampling, chroma_sample_position) = map_y4m_color_space(color_space);
        let framerate = self.inner.get_framerate();
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

    fn read_video_frame<T: Pixel>(&mut self) -> Option<Frame<T>> {
        let bit_depth = self.inner.get_bit_depth();
        let color_space = self.inner.get_colorspace();
        let (chroma_sampling, chroma_sample_pos) = map_y4m_color_space(color_space);
        let width = self.inner.get_width();
        let height = self.inner.get_height();
        let bytes = self.inner.get_bytes_per_sample();
        self.inner.read_frame().ok().map(|frame| {
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

            f
        })
    }

    fn get_bit_depth(&self) -> usize {
        self.inner.get_bit_depth()
    }
}
