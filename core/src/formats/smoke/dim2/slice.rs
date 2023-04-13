use crate::common::series::TimeSeries2;
use crate::formats::read_ext::{ReadExt, U32Ext};
pub use crate::formats::smoke::parse_err::Error;
use crate::geom::{Bounds3I, Dim3D, Vec2, Vec2U, Vec3I};
use byteorder::ReadBytesExt;
use get_size::GetSize;
use ndarray::{Array1, Array2, Array3, Axis};
use std::io::Read;
use tracing::instrument;

use super::slice_frame::SliceFrame;

#[derive(Debug, GetSize)]
pub struct SliceInfo {
    pub bounds: Bounds3I,
    pub flat_dim: Dim3D,
    pub quantity: String,
    pub short_name: String,
    pub units: String,
}

#[derive(Debug, GetSize)]
pub struct Slice {
    pub info: SliceInfo,
    pub data: TimeSeries2,
}

impl SliceInfo {
    pub fn dim_i(&self) -> Dim3D {
        if self.flat_dim == Dim3D::X {
            Dim3D::Y
        } else {
            Dim3D::X
        }
    }

    pub fn dim_j(&self) -> Dim3D {
        if self.flat_dim == Dim3D::Z {
            Dim3D::Y
        } else {
            Dim3D::Z
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

    pub fn area(&self) -> Vec2U {
        Vec2U::new(self.dim_i_len(), self.dim_j_len())
    }
}

impl Slice {
    #[instrument(skip(rdr))]
    pub fn from_reader(mut rdr: impl Read) -> Result<Slice, Error> {
        // TODO: Should the underlying error be annotated with added context?
        let quantity = rdr.read_fortran_string()?;
        // TODO: Not technically neccessary double allocation, once in read_fortran_string, once here
        let quantity = quantity.trim().to_string();

        let short_name = rdr.read_fortran_string()?;
        let short_name = short_name.trim().to_string();

        // TODO: Why is units plural? Should it be? Can there be multiple units? (name taken from the C# version)
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

            let min = Vec3I::new(vals[0], vals[2], vals[4]);
            let max = Vec3I::new(vals[1], vals[3], vals[5]);
            let max = max + Vec3I::ONE;

            Bounds3I::new(min, max)
        };

        // Size of the bounds
        rdr.read_fixed_u32(6 * 4)?;

        let volume = bounds.area().x * bounds.area().y * bounds.area().z;

        let flat_dim = bounds.area().enumerate().find(|(_, x)| *x == 1);
        let flat_dim = match flat_dim {
            Some((dim, _)) => dim,
            None => {
                return Err(Error::BadBoundsSize);
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
                Err(Error::NoBlocks) => {
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
    fn from_frames(info: &SliceInfo, frames: Vec<SliceFrame>) -> Result<Self, Error> {
        let area = info.area();
        // TODO: Store usize directly?
        let area = Vec2::new(area.x.try_into_usize()?, area.y.try_into_usize()?);

        let mut time_arr = Array1::zeros(frames.len());
        let mut values_arr = Array3::zeros((frames.len(), area.x, area.y));

        for (i, frame) in frames.into_iter().enumerate() {
            time_arr[i] = frame.time.value;
            // dbg!(&frame.values);
            values_arr
                .index_axis_mut(Axis(0), 0)
                .assign(&Array2::from_shape_vec((area.x, area.y), frame.values)?);
        }

        Ok(TimeSeries2::new(
            info.short_name.clone(),
            info.units.clone(),
            time_arr.into(),
            values_arr.into(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_example() {
        let data = include_bytes!("../../../../../demo-house/DemoHaus2_0004_39.sf");
        let slice = Slice::from_reader(&data[..]).unwrap();
        // dbg!(&slice);

        assert_eq!(slice.info.quantity, "SOOT OPTICAL DENSITY");
        assert_eq!(slice.info.short_name, "OD_C0.9H0.1");
        assert_eq!(slice.info.units, "1/m");

        assert_eq!(slice.info.bounds.min, Vec3I::new(0, 0, 43));
        assert_eq!(slice.info.bounds.max, Vec3I::new(34, 34, 44));

        assert_eq!(slice.info.flat_dim, Dim3D::Z);
    }
}
