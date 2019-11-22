use crate::video::decode::Decoder;
use crate::video::pixel::CastFromPrimitive;
use crate::video::pixel::Pixel;
use crate::video::{ChromaSamplePosition, ChromaSampling, FrameInfo, PlaneData};
use std::io::Read;
use std::{cmp, mem};

fn get_chroma_sampling<R: Read>(
    dec: &y4m::Decoder<'_, R>,
) -> (ChromaSampling, ChromaSamplePosition) {
    use crate::video::ChromaSamplePosition::*;
    use crate::video::ChromaSampling::*;
    use y4m::Colorspace::*;
    match dec.get_colorspace() {
        Cmono => (Cs400, Unknown),
        C420jpeg => (Cs420, Bilateral),
        C420paldv => (Cs420, Interpolated),
        C420mpeg2 => (Cs420, Vertical),
        C420 | C420p10 | C420p12 => (Cs420, Colocated),
        C422 | C422p10 | C422p12 => (Cs422, Vertical),
        C444 | C444p10 | C444p12 => (Cs444, Colocated),
    }
}

fn copy_from_raw_u8<T: Pixel>(source: &[u8]) -> Vec<T> {
    match mem::size_of::<T>() {
        1 => source.iter().map(|byte| T::cast_from(*byte)).collect(),
        2 => {
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

impl<R: Read + Send> Decoder for y4m::Decoder<'_, R> {
    fn read_video_frame<T: Pixel>(&mut self) -> Result<FrameInfo<T>, ()> {
        let bit_depth = self.get_bit_depth();
        let (chroma_sampling, chroma_sample_pos) = get_chroma_sampling(self);
        let luma_width = self.get_width();
        let luma_height = self.get_height();
        let (chroma_width, chroma_height) =
            chroma_sampling.get_chroma_dimensions(luma_width, luma_height);

        self.read_frame()
            .map(|frame| FrameInfo {
                bit_depth,
                chroma_sampling,
                planes: [
                    PlaneData {
                        width: luma_width,
                        height: luma_height,
                        data: copy_from_raw_u8(frame.get_y_plane()),
                    },
                    convert_chroma_data(
                        PlaneData {
                            width: chroma_width,
                            height: chroma_height,
                            data: copy_from_raw_u8(frame.get_u_plane()),
                        },
                        chroma_sample_pos,
                        bit_depth,
                    ),
                    convert_chroma_data(
                        PlaneData {
                            width: chroma_width,
                            height: chroma_height,
                            data: copy_from_raw_u8(frame.get_v_plane()),
                        },
                        chroma_sample_pos,
                        bit_depth,
                    ),
                ],
            })
            .map_err(|_| ())
    }

    fn read_specific_frame<T: Pixel>(&mut self, frame_number: usize) -> Result<FrameInfo<T>, ()> {
        let mut frame_no = 0;
        while frame_no <= frame_number {
            let frame = self.read_video_frame();
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
fn convert_chroma_data<T: Pixel>(
    plane_data: PlaneData<T>,
    chroma_pos: ChromaSamplePosition,
    bit_depth: usize,
) -> PlaneData<T> {
    if chroma_pos != ChromaSamplePosition::Vertical {
        // TODO: Also convert Interpolated chromas
        return plane_data;
    }
    let mut output_data = vec![T::cast_from(0u8); plane_data.data.len()];
    let width = plane_data.width;
    let height = plane_data.height;
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
    PlaneData {
        width,
        height,
        data: output_data,
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
