use av_data::params::MediaKind;
use av_format::buffer::Buffered;
use av_format::common::GlobalInfo;
use av_format::demuxer::Demuxer;
use av_ivf::demuxer::IvfDemuxer;
use av_metrics::v_frame::pixel::Pixel;
use av_metrics::video::{
    ChromaSamplePosition, ChromaSampling, Decoder, FrameInfo, Rational, VideoDetails,
};

pub struct IvfDecoder<'a> {
    demuxer: IvfDemuxer,
    headers: VideoDetails,
    #[allow(clippy::borrowed_box)]
    reader: &'a Box<dyn Buffered>,
}

impl<'a> IvfDecoder<'a> {
    #[allow(clippy::borrowed_box)]
    pub fn new(reader: &'a Box<dyn Buffered>) -> Self {
        let mut demuxer = IvfDemuxer::new();
        let mut info = GlobalInfo {
            duration: None,
            timebase: None,
            streams: Vec::new(),
        };
        demuxer
            .read_headers(reader, &mut info)
            .expect("Failed to read IVF headers");
        let video_info = match info.streams[0].params.kind {
            Some(MediaKind::Video(ref info)) => info,
            _ => panic!("Failed to find video stream in IVF container"),
        };
        let headers = VideoDetails {
            width: video_info.width,
            height: video_info.height,
            // FIXME: THE FUCKING DEMUXER DOESN'T PARSE ANY OF THIS.
            bit_depth: 8,
            chroma_sampling: ChromaSampling::Cs420,
            chroma_sample_position: ChromaSamplePosition::Unknown,
            time_base: info
                .timebase
                .map(|tb| Rational {
                    num: *tb.numer() as u64,
                    den: *tb.denom() as u64,
                })
                .unwrap_or_else(|| Rational { num: 30, den: 1 }),
            luma_padding: 0,
        };
        IvfDecoder {
            demuxer,
            headers,
            reader,
        }
    }
}

impl<'a> Decoder for IvfDecoder<'a> {
    fn read_video_frame<T: Pixel>(&mut self, cfg: &VideoDetails) -> Result<FrameInfo<T>, ()> {
        unimplemented!()
    }

    fn read_specific_frame<T: Pixel>(&mut self, frame_number: usize) -> Result<FrameInfo<T>, ()> {
        unimplemented!()
    }

    fn get_bit_depth(&self) -> usize {
        unimplemented!()
    }

    fn get_video_details(&self) -> VideoDetails {
        self.headers
    }
}
