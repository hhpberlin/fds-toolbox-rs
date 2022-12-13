use std::io::{Read, Seek, SeekFrom};
use crate::common::series::Series2;
use crate::formats::slice::slice_frame::SliceFrame;
use crate::geom::bounds3int::{Bounds3I, Dimension3D};
use byteorder::ReadBytesExt;
use strum::IntoEnumIterator;

use super::slice_frame::SliceFrameErr;

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
    fn new(reader: &mut (impl Read + Seek)) -> Result<Slice, SliceFrameErr>
    {
        let mut slice = Slice::default();
        
        //let _ = reader.read_to_string(&mut slice.quantity);
        let _ = reader.seek(SeekFrom::Current(1));
        //let _ = reader.read_to_string(&mut slice.short_name);
        let _ = reader.seek(SeekFrom::Current(1));
        //let _ = reader.read_to_string(&mut slice.units);
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
        
        let block = slice.bounds.area().x *slice.bounds.area().y *slice.bounds.area().z;
        for i in Dimension3D::iter() {
            if slice.bounds.area()[i] == 1
            {
                slice.flat_dimension = i;
                slice.flat_dimension_position = slice.bounds.min[i];
                slice.dimension_i = if i == Dimension3D::X {Dimension3D::Y} else { Dimension3D::X};
                slice.dimension_j = if i == Dimension3D::Z {Dimension3D::Y} else { Dimension3D::Z};
                break;
            }
            
        }
        
        Ok(slice)
    }
}


