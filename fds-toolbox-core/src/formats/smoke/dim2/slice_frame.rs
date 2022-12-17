use std::io::Read;

use crate::formats::{
    read_ext::{ReadBlockErr, ReadExt, U32Ext},
    smoke::parse_err::ParseErr,
};

use uom::si::{f32::Time, time::second};

use super::slice::SliceInfo;
use byteorder::ReadBytesExt;

#[derive(Default)]
pub struct SliceFrame {
    pub time: Time,
    pub values: Vec<f32>,
}

impl SliceFrame {
    pub fn from_reader(
        mut rdr: impl Read,
        slice: &SliceInfo,
        volume: u32,
    ) -> Result<SliceFrame, ParseErr> {
        rdr.read_fixed_u32(4)
            // TODO: Should IO Error really be discarded?
            .map_err(|_x| ParseErr::NoBlocks)?;

        let time = Time::new::<second>(rdr.read_f32::<byteorder::LittleEndian>()?);

        rdr.read_fixed_u32(4)?;

        let block_size = rdr.read_u32::<byteorder::LittleEndian>()?;

        // 4 bytes per value = 4 * volume = block_size
        if volume * 4 != block_size {
            return Err(ParseErr::BadFrameSize {
                read: block_size,
                expected: volume,
            });
        }

        let block_size = block_size.try_into_usize()?;

        let len_i = slice.bounds.area()[slice.dim_i()];
        let len_j = slice.bounds.area()[slice.dim_j()];

        let len_i = len_i.try_into_usize()?;
        let len_j = len_j.try_into_usize()?;

        let mut values = vec![0.0; volume.try_into_usize()?];
        debug_assert_eq!(values.len(), len_i * len_j);

        rdr.read_f32_into::<byteorder::LittleEndian>(&mut values[..])?;

        let block_size_postfix = rdr.read_u32::<byteorder::LittleEndian>()?;
        let block_size_postfix = block_size_postfix.try_into_usize()?;

        if block_size != block_size_postfix {
            return Err(ParseErr::BadBlock(ReadBlockErr::MismatchedPostfixLength(
                block_size,
                block_size_postfix,
            )));
        }

        Ok(SliceFrame { time, values })
    }
}
