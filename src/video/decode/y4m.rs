use crate::video::decode::Decoder;
use crate::video::pixel::CastFromPrimitive;
use crate::video::pixel::Pixel;
use crate::video::{ChromaSampling, FrameInfo, PlaneData};
use std::io::Read;
use std::mem;

fn get_chroma_sampling<R: Read>(dec: &y4m::Decoder<'_, R>) -> ChromaSampling {
    use crate::video::ChromaSampling::*;
    use y4m::Colorspace::*;
    match dec.get_colorspace() {
        Cmono => Cs400,
        C420jpeg | C420paldv | C420mpeg2 | C420 | C420p10 | C420p12 => Cs420,
        C422 | C422p10 | C422p12 => Cs422,
        C444 | C444p10 | C444p12 => Cs444,
    }
}

pub fn copy_from_raw_u8<T: Pixel>(source: &[u8], pixel_width: usize) -> Vec<T> {
    match pixel_width {
        1 => {
            assert!(mem::size_of::<T>() == 1);
            source.iter().map(|byte| T::cast_from(*byte)).collect()
        }
        2 => {
            assert!(mem::size_of::<T>() == 2);
            let mut output = Vec::with_capacity(source.len() / 2);
            for bytes in source.chunks(2) {
                output.push(T::cast_from(
                    u16::cast_from(bytes[1]) << 8 | u16::cast_from(bytes[0]),
                ));
            }
            output
        }
        _ => unreachable!(),
    }
}

impl<T: Pixel, R: Read> Decoder<T> for y4m::Decoder<'_, R> {
    fn read_video_frame(&mut self) -> Result<FrameInfo<T>, ()> {
        let bit_depth = self.get_bit_depth();
        let chroma_sampling = get_chroma_sampling(self);
        let luma_width = self.get_width();
        let luma_height = self.get_height();
        let (chroma_width, chroma_height) =
            chroma_sampling.get_chroma_dimensions(luma_width, luma_height);
        let pixel_width = (bit_depth > 8) as usize + 1;

        self.read_frame()
            .map(|frame| FrameInfo {
                bit_depth,
                chroma_sampling,
                planes: [
                    PlaneData {
                        width: luma_width,
                        height: luma_height,
                        data: copy_from_raw_u8(frame.get_y_plane(), pixel_width),
                    },
                    PlaneData {
                        width: chroma_width,
                        height: chroma_height,
                        data: copy_from_raw_u8(frame.get_u_plane(), pixel_width),
                    },
                    PlaneData {
                        width: chroma_width,
                        height: chroma_height,
                        data: copy_from_raw_u8(frame.get_v_plane(), pixel_width),
                    },
                ],
            })
            .map_err(|_| ())
    }
}
