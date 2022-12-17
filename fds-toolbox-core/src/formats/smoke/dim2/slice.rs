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
    pub fn from_reader(mut rdr: impl Read) -> Result<Slice, ParseErr> {
        // TODO: Should the underlying error be annotated with added context?
        let quantity = rdr.read_fortran_string()?;
        // TODO: Not technically neccessary double allocation
        let quantity = quantity.trim().to_string();

        let short_name = rdr.read_fortran_string()?;
        let short_name = short_name.trim().to_string();

        let units = rdr.read_fortran_string()?;
        let units = units.trim().to_string();

        // Size of the bounds
        rdr.read_fixed_u32(6 * 4)?;

        let bounds = {
            let vals = [
                rdr.read_i32::<byteorder::LittleEndian>()?,
                rdr.read_i32::<byteorder::LittleEndian>()?,
                rdr.read_i32::<byteorder::LittleEndian>()?,
                rdr.read_i32::<byteorder::LittleEndian>()?,
                rdr.read_i32::<byteorder::LittleEndian>()?,
                rdr.read_i32::<byteorder::LittleEndian>()?,
            ];

            let min = Point3I::new(vals[0], vals[2], vals[4]);
            let max = Point3I::new(vals[1], vals[3], vals[5]);
            let max = max + Point3I::ONE;

            Bounds3I::new(min, max)
        };

        // Size of the bounds
        rdr.read_fixed_u32(6 * 4)?;

        let volume = bounds.area().x * bounds.area().y * bounds.area().z;

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

        dbg!(&slice_info);

        loop {
            match SliceFrame::from_reader(&mut rdr, &slice_info, volume) {
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
