
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