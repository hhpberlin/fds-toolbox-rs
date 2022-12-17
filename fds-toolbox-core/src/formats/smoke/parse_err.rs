use thiserror::Error;

use crate::formats::read_ext::ReadStrErr;

#[derive(Error, Debug)]
pub enum ParseErr {
    #[error("Bad block")]
    BadBlock,
    #[error("Bad string: {0}")]
    BadString(#[from] ReadStrErr),
    #[error("Bad bounds size, no dimension was size 1")]
    BadBoundsSize,
    #[error("Bad block size: {0}")]
    BadBlockSize(usize),
    #[error("I/O error: {0}")]
    IoErr(#[from] std::io::Error),
    #[error("EOF")]
    NoBlocks,
}
