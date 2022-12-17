use crate::common::series::{TimeSeries2, Series1};
use crate::formats::read_ext::{ReadExt, U32Ext};
use crate::formats::smoke::parse_err::ParseErr;
use crate::geom::{Bounds3I, Dimension3D, Point3I, Point2U, Point2};
use byteorder::ReadBytesExt;
use ndarray::{Array3, s, Array2, Array1, Axis};
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
    pub data: TimeSeries2,
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

    pub fn flat_dim_pos(&self) -> u32 {
        self.bounds.area()[self.flat_dim]
    }

    pub fn dim_i_len(&self) -> u32 {
        self.bounds.area()[self.dim_i()]
    }

    pub fn dim_j_len(&self) -> u32 {
        self.bounds.area()[self.dim_j()]
    }

    pub fn area(&self) -> Point2U {
        Point2U::new(self.dim_i_len(), self.dim_j_len())
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

        // TODO: Why is units plural? Should it be? Can there be multiple units?
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

        loop {
            match SliceFrame::from_reader(&mut rdr, &slice_info, volume) {
                Ok(frame) => {
                    frames.push(frame);
                }
                Err(ParseErr::NoBlocks) => {
                    // TODO: Avoid copying all the data here?
                    //       Instead maybe write directly to a shared Vec from the beginning
                    //       Although resizing the Vec might just be doing the same thing
                    let data = TimeSeries2::from_frames(&slice_info, frames)?;
                    return Ok(Slice {
                        data,
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
    fn from_frames(info: &SliceInfo, frames: Vec<SliceFrame>) -> Result<Self, ParseErr> {
        let area = info.area();
        // TODO: Store usize directly?
        let area = Point2::new(area.x.try_into_usize()?, area.y.try_into_usize()?);

        let mut time_arr = Array1::zeros(frames.len());
        let mut values_arr = Array3::zeros((frames.len(), area.x, area.y));

        for (i, frame) in frames.into_iter().enumerate() {
            time_arr[i] = frame.time.value;
            values_arr.index_axis_mut(Axis(0), 0).assign(&Array2::from_shape_vec((area.x, area.y), frame.values)?);
        }

        Ok(TimeSeries2::new(info.short_name.clone(), info.units.clone(), time_arr.into(), values_arr.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_example() {
        let data = include_bytes!("../../../../../demo-house/DemoHaus2_0004_39.sf");
        let slice = Slice::from_reader(&data[..]).unwrap();

        assert_eq!(slice.info.quantity, "SOOT OPTICAL DENSITY");
        assert_eq!(slice.info.short_name, "OD_C0.9H0.1");
        assert_eq!(slice.info.units, "1/m");

        assert_eq!(slice.info.bounds.min, Point3I::new(0, 0, 43));
        assert_eq!(slice.info.bounds.max, Point3I::new(34, 34, 44));

        assert_eq!(slice.info.flat_dim, Dimension3D::Z);
    }
}