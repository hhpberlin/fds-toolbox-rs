use std::io::Read;
use byteorder::ReadBytesExt;
use crate::formats::{
    read_ext::{ReadBlockErr, ReadExt, U32Ext},
    smoke::parse_err::Error,
};

use crate::geom::Vec3I;
use crate::formats::smoke::dim3::slice_frame::SliceFrame;

pub struct  S3D {
    pub Size: Vec3I,
    pub MinValues: Vec3I, //should be a byte array, I hate rust
    pub MaxValues: Vec3I, //should be a byte array, I hate rust
    pub Frames: Vec<SliceFrame>,
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
        
        let Size = Vec3I::new(x, y, z);
        let mut ret = S3D {
            Size: Size,
            MinValues: Size,
            MaxValues: Size,
            Frames: Vec::new(),
        };
        loop {
            match SliceFrame::from_read(&rdr, &ret) {
                Ok(frame) => {
                    ret.Frames.push(frame);
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