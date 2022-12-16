use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseErr {
    #[error("Bad block")]
    BadBlock,
    #[error("Bad bounds size, no dimension was size 1")]
    BadBoundsSize,
    #[error("Bad block size: {0}")]
    BadBlockSize(usize),
    #[error("I/O error: {0}")]
    IoErr(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8Err(#[from] std::string::FromUtf8Error),
    #[error("EOF")]
    NoBlocks,
}
