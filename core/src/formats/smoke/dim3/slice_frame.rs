use crate::formats::smoke::parse_err::Error;

use std::io::Read;

use crate::formats::smoke::dim3::s3d::S3D;

#[derive(Default)]
pub struct SliceFrame {}

impl SliceFrame {
    pub fn from_read(_rdr: &impl Read, _obj: &S3D) -> Result<SliceFrame, Error> {
        Ok(SliceFrame {})
    }
}
