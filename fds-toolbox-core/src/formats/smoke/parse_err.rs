use std::num::TryFromIntError;

use thiserror::Error;

use crate::formats::read_ext::{ReadStrErr, ReadBlockErr, ReadValErr};

#[derive(Error, Debug)]
pub enum ParseErr {
    #[error("Bad block")]
    BadBlock(#[from] ReadBlockErr),
    #[error("Bad string: {0}")]
    BadString(#[from] ReadStrErr),
    #[error("Bad magic number: {0}")]
    BadMagicNumber(#[from] ReadValErr<u32>),

    #[error("Bad bounds size, no dimension was size 1")]
    BadBoundsSize,

    #[error("Bad block size: read {read}, expected {expected}")]
    BadFrameSize { read: u32, expected: u32 },

    #[error("Couldn't convert size of {0} bytes to native pointer size: {1}")]
    InvalidLength(u32, TryFromIntError),

    #[error("I/O error: {0}")]
    IoErr(#[from] std::io::Error),
    #[error("EOF")]
    NoBlocks,
}
