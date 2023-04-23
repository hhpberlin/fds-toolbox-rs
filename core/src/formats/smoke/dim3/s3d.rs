use crate::formats::{read_ext::ReadExt, smoke::parse_err::Error};
use byteorder::ReadBytesExt;
use std::io::Read;

use crate::formats::smoke::dim3::slice_frame::SliceFrame;
use crate::geom::Vec3I;

pub struct S3D {
    pub size: Vec3I,
    pub min_values: Vec3I, //should be a byte array, I hate rust
    pub max_values: Vec3I, //should be a byte array, I hate rust
    pub frames: Vec<SliceFrame>,
}

//add quantity enum???

impl S3D {
    pub fn from_read(mut rdr: impl Read) -> Result<S3D, Error> {
        rdr.read_fixed_u32(4)?;
        rdr.read_fixed_u32(4)?;
        rdr.read_fixed_u32(4)?;
        rdr.read_fixed_u32(4)?;

        let x: i32 = 1 + rdr.read_i32::<byteorder::LittleEndian>()?;
        rdr.read_fixed_u32(4)?;
        let y: i32 = 1 + rdr.read_i32::<byteorder::LittleEndian>()?;
        rdr.read_fixed_u32(4)?;
        let z: i32 = 1 + rdr.read_i32::<byteorder::LittleEndian>()?;
        rdr.read_fixed_u32(4)?;

        let size = Vec3I::new(x, y, z);
        let mut ret = S3D {
            size,
            min_values: size,
            max_values: size,
            frames: Vec::new(),
        };
        loop {
            match SliceFrame::from_read(&rdr, &ret) {
                Ok(frame) => {
                    ret.frames.push(frame);
                }
                Err(Error::NoBlocks) => {
                    break;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(ret)
    }
}
