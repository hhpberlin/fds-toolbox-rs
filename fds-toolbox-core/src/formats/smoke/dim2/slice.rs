use crate::common::series::Series2;
use crate::formats::smoke::parse_err::ParseErr;
use crate::geom::bounds3int::{Bounds3I, Dimension3D};
use byteorder::ReadBytesExt;
use std::io::{Read, Seek, SeekFrom, self};
use strum::IntoEnumIterator;

use super::slice_frame::SliceFrame;

#[derive(Default)]
pub struct Slice {
    pub min_value: f32,
    pub max_value: f32,
    pub frames: Series2,
    pub bounds: Bounds3I,
    pub flat_dimension: Dimension3D,
    pub flat_dimension_position: i32,
    pub dimension_i: Dimension3D,
    pub dimension_j: Dimension3D,
    pub quantity: String,
    pub short_name: String,
    pub units: String,
}

impl Slice {
    fn new(mut reader: impl Read + Seek) -> Result<Slice, ParseErr> {
        let mut slice = Slice::default();

        slice.quantity = reader.read_string()?;
        let _ = reader.seek(SeekFrom::Current(1));
        slice.short_name = reader.read_string()?;
        let _ = reader.seek(SeekFrom::Current(1));
        slice.units = reader.read_string()?;
        let _ = reader.seek(SeekFrom::Current(2));

        //let a = reader.read_i32::<byteorder::BigEndian>()?;

        slice.bounds = Bounds3I::new(
            reader.read_i32::<byteorder::BigEndian>()?,
            reader.read_i32::<byteorder::BigEndian>()?,
            reader.read_i32::<byteorder::BigEndian>()?,
            (reader.read_i32::<byteorder::BigEndian>()?) + 1,
            (reader.read_i32::<byteorder::BigEndian>()?) + 1,
            (reader.read_i32::<byteorder::BigEndian>()?) + 1,
        );
        let _ = reader.seek(SeekFrom::Current(2));

        let block = slice.bounds.area().x * slice.bounds.area().y * slice.bounds.area().z;
        for i in Dimension3D::iter() {
            if slice.bounds.area()[i] == 1 {
                slice.flat_dimension = i;
                slice.flat_dimension_position = slice.bounds.min[i];
                slice.dimension_i = if i == Dimension3D::X {
                    Dimension3D::Y
                } else {
                    Dimension3D::X
                };
                slice.dimension_j = if i == Dimension3D::Z {
                    Dimension3D::Y
                } else {
                    Dimension3D::Z
                };
                break;
            }
        }

        slice.min_value = f32::INFINITY;
        slice.max_value = f32::NEG_INFINITY;

        let mut frames = Vec::new();

        loop {
            match SliceFrame::new(&mut reader, &slice, block) {
                Ok(frame) => {
                    slice.max_value = slice.max_value.max(frame.max_value);
                    slice.min_value = slice.min_value.min(frame.min_value);
                    frames.push(frame);
                }
                Err(ParseErr::NoBlocks) => {
                    slice.frames = Series2::from_data(frames);
                    return Ok(slice);
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok(slice)
    }
}

pub trait ReadExt {
    fn read_string(&mut self) -> Result<String, ParseErr>;
    fn skip<const N: usize>(&mut self, n: usize) -> Result<(), io::Error>;
}

impl<T: Read> ReadExt for T {
    fn read_string(&mut self) -> Result<String, ParseErr> {
        let mut buf: Vec<u8> = vec![0u8; self.read_i32::<byteorder::BigEndian>()? as usize];
        self.read_exact(&mut buf)?;
        // TODO: Should the underlying error be propagated?
        String::from_utf8(buf).map_err(|_| { ParseErr::BadBlock })
    }

    fn skip<const N: usize>(&mut self, n: usize) -> Result<(), io::Error> {
        self.read_exact(&mut [0; N])
    }
}

impl Series2 {
    pub fn from_data(_data: Vec<SliceFrame>) -> Self {
        todo!();
    }
}
