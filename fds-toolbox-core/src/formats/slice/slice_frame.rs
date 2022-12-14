use std::io::{Read, Seek, SeekFrom, self};

use thiserror::Error;
use uom::si::{f32::Time, time::second};

use super::slice::Slice;
use byteorder::ReadBytesExt;

#[derive(Default)]
pub struct SliceFrame {
    pub time: Time,
    pub values: Vec<Vec<f32>>,
    pub min_value: f32,
    pub max_value: f32,
}

#[derive(Error, Debug)]
pub enum SliceFrameErr {
    #[error("Bad block")]
    BadBlock,
    #[error("Bad block size: {0}")]
    BadBlockSize(usize),
    #[error("I/O error: {0}")]
    IoErr(std::io::Error),
    #[error("EOF")]
    NoBlocks,
}

impl From<io::Error> for SliceFrameErr
{
    fn from(err: io::Error) -> Self {
        SliceFrameErr::IoErr(err)
    }
}

impl SliceFrame {
    pub fn new(reader: &mut (impl Read + Seek), slice: &Slice, block: i32) -> Result<SliceFrame, SliceFrameErr> {
        let mut ret: SliceFrame = SliceFrame {
            time: Time::new::<second>(reader.read_f32::<byteorder::BigEndian>().map_err(SliceFrameErr::NoBlocks)),
            values: vec![vec![0.; slice.bounds.area()[slice.dimension_j]as usize];slice.bounds.area()[slice.dimension_i] as usize],
            min_value: f32::INFINITY,
            max_value: f32::NEG_INFINITY,
        };
        let _ = reader.seek(SeekFrom::Current(1));

        let block_size = reader.read_i32::<byteorder::BigEndian>();
        match block_size {
            Err(r) => return Err(SliceFrameErr::IoErr(r)),
            Ok(blk) => {
                if block * 4 != blk {
                    return Err(SliceFrameErr::BadBlock);
                }
                for j in 0..slice.bounds.area()[slice.dimension_i]  {
                    for k in 0..slice.bounds.area()[slice.dimension_j]  {
                        let value = reader.read_f32::<byteorder::BigEndian>()?;
                        ret.values[j as usize][k as usize] = value;
                        ret.min_value = value.min(ret.min_value);
                        ret.max_value = value.max(ret.max_value);                    }
                }
                let _ = reader.seek(SeekFrom::Current(1));
            }
        }
        Ok(ret)
    }
}
