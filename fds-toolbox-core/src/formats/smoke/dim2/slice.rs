use crate::common::series::TimeSeries2;
use crate::formats::read_ext::ReadExt;
use crate::formats::smoke::parse_err::ParseErr;
use crate::geom::{Bounds3I, Dimension3D, Point3I};
use byteorder::ReadBytesExt;
use std::io::Read;

use super::slice_frame::SliceFrame;

#[derive(Debug)]
pub struct SliceInfo {
    pub bounds: Bounds3I,
    pub flat_dim: Dimension3D,
    pub quantity: String,
    pub short_name: String,
    pub units: String,
}

#[derive(Debug)]
pub struct Slice {
    pub info: SliceInfo,
    pub frames: TimeSeries2,
}

impl SliceInfo {
    pub fn dim_i(&self) -> Dimension3D {
        if self.flat_dim == Dimension3D::X {
            Dimension3D::Y
        } else {
            Dimension3D::X
        }
    }

    pub fn dim_j(&self) -> Dimension3D {
        if self.flat_dim == Dimension3D::Z {
            Dimension3D::Y
        } else {
            Dimension3D::Z
        }
    }

    pub fn flat_dim_len(&self) -> u32 {
        self.bounds.area()[self.flat_dim]
    }
}

impl Slice {
    pub fn from_reader(mut reader: impl Read) -> Result<Slice, ParseErr> {
        // TODO: Should the underlying error be annotated with added context?
        let quantity = reader.read_string()?;
        reader.skip(4)?;
        let short_name = reader.read_string()?;
        reader.skip(4)?;
        let units = reader.read_string()?;
        reader.skip(2 * 4)?;

        //let a = reader.read_i32::<byteorder::BigEndian>()?;

        let min = Point3I::new(
            reader.read_i32::<byteorder::LittleEndian>()?,
            reader.read_i32::<byteorder::LittleEndian>()?,
            reader.read_i32::<byteorder::LittleEndian>()?,
        );

        let max = Point3I::new(
            reader.read_i32::<byteorder::LittleEndian>()? + 1,
            reader.read_i32::<byteorder::LittleEndian>()? + 1,
            reader.read_i32::<byteorder::LittleEndian>()? + 1,
        );

        let bounds = Bounds3I::new(min, max);

        reader.skip(2 * 4)?;

        dbg!(bounds);
        let block_size = bounds.area().x * bounds.area().y * bounds.area().z;

        let flat_dim = bounds.area().enumerate().find(|(_, x)| *x == 1);
        let flat_dim = match flat_dim {
            Some((dim, _)) => dim,
            None => {
                return Err(ParseErr::BadBoundsSize);
            }
        };

        let slice_info = SliceInfo {
            bounds,
            flat_dim,
            quantity,
            short_name,
            units,
        };

        let mut frames = Vec::new();

        loop {
            match SliceFrame::new(&mut reader, &slice_info, block_size) {
                Ok(frame) => {
                    frames.push(frame);
                }
                Err(ParseErr::NoBlocks) => {
                    let frames = TimeSeries2::from_data(frames);
                    return Ok(Slice {
                        frames,
                        info: slice_info,
                    });
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
    }
}

impl TimeSeries2 {
    pub fn from_data(_data: Vec<SliceFrame>) -> Self {
        todo!();
    }
}
