use crate::formats::{
    read_ext::{ReadBlockErr, ReadExt, U32Ext},
    smoke::parse_err::Error,
};
use byteorder::ReadBytesExt;
use std::io::Read;

use crate::formats::smoke::dim3::s3d::S3D;
use crate::geom::Vec3I;

#[derive(Default)]
pub struct SliceFrame {}

impl SliceFrame {
    pub fn from_read(mut rdr: &(impl Read), obj: &S3D) -> Result<SliceFrame, Error> {
        Ok(SliceFrame {})
    }
}
