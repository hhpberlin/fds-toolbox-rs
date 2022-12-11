use std::io::Read;

use byteorder::ReadBytesExt;
use thiserror::Error;
use uom::si::{f32::Time, time::second};

use super::slice::Slice;

pub struct SliceFrame {
    time: Time,
    values: Vec<Vec<f32>>,
    min_value: f32,
    max_value: f32,
}

#[derive(Error, Debug)]
pub enum SliceFrameErr {
    #[error("Bad block")]
    BadBlock,
    #[error("Bad block size: {0}")]
    BadBlockSize(usize),
    #[error("I/O error: {0}")]
    IoErr(std::io::Error),
}

impl SliceFrame {
    fn new(reader: &mut impl Read, slice: Slice, block: i32) -> Result<SliceFrame, SliceFrameErr> {
        let mut ret: SliceFrame = SliceFrame {
            time: Time::new::<second>(reader.read_f32()?),
            values: vec![],
            min_value: f32::MAX,
            max_value: f32::MIN,
        };
        let _ = reader.read_i32()?;

        let block_size = reader.read_i32()?;
        match block_size {
            None => return Err(SliceFrameErr::BadBlockSize(block_size)),
            Some(blk) => {
                if block * 4 != blk {
                    return Err(SliceFrameErr::BadBlock);
                }
            }
        }
        Ok(ret)
    }
}
