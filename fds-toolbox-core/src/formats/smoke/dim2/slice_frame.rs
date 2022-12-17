use std::io::Read;

use crate::formats::{read_ext::ReadExt, smoke::parse_err::ParseErr};

use uom::si::{f32::Time, time::second};

use super::slice::SliceInfo;
use byteorder::ReadBytesExt;

#[derive(Default)]
pub struct SliceFrame {
    pub time: Time,
    pub values: Vec<Vec<f32>>,
}

impl SliceFrame {
    pub fn from_reader(
        mut reader: impl Read,
        slice: &SliceInfo,
        block: u32,
    ) -> Result<SliceFrame, ParseErr> {
        let mut ret: SliceFrame = SliceFrame {
            time: Time::new::<second>(
                reader
                    .read_f32::<byteorder::LittleEndian>()
                    .map_err(|_x| ParseErr::NoBlocks)?, // TODO: Should IO Error really be discarded?
            ),
            values: vec![
                vec![0.; slice.bounds.area()[slice.dim_j()] as usize];
                slice.bounds.area()[slice.dim_i()] as usize
            ],
        };
        reader.skip(4)?;

        let block_size = reader.read_u32::<byteorder::LittleEndian>()?;
        if block * 4 != block_size {
            return Err(ParseErr::BadBlock);
        }
        for j in 0..slice.bounds.area()[slice.dim_i()] {
            for k in 0..slice.bounds.area()[slice.dim_j()] {
                let value = reader.read_f32::<byteorder::LittleEndian>()?;
                ret.values[j as usize][k as usize] = value;
            }
        }
        reader.skip(4)?;

        Ok(ret)
    }
}
