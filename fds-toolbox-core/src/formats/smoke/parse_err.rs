use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseErr {
    #[error("Bad block")]
    BadBlock,
    #[error("Bad block size: {0}")]
    BadBlockSize(usize),
    #[error("I/O error: {0}")]
    IoErr(std::io::Error),
    #[error("EOF")]
    NoBlocks,
}

impl From<io::Error> for ParseErr {
    fn from(err: io::Error) -> Self {
        ParseErr::IoErr(err)
    }
}
