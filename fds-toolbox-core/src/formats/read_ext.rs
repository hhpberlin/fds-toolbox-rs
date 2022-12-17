use std::io::{self, Read};

use byteorder::ReadBytesExt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReadStrErr {
    #[error("IO error {0}")]
    Io(#[from] io::Error),
    #[error("IO error {0}, expected to read {1} bytes")]
    IoBuf(io::Error, usize),
    #[error("UTF-8 error {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub trait ReadExt {
    fn read_string(&mut self) -> Result<String, ReadStrErr>;
    fn skip(&mut self, n: usize) -> Result<(), io::Error>;
}

impl<T: Read> ReadExt for T {
    fn read_string(&mut self) -> Result<String, ReadStrErr> {
        let len = self
            .read_u32::<byteorder::BigEndian>()?;
        let len = len as usize;

        let mut buf: Vec<u8> = vec![0u8; len];
        self.read_exact(&mut buf).map_err(|err| ReadStrErr::IoBuf(err, len))?;
        
        Ok(String::from_utf8(buf)?)
    }

    fn skip(&mut self, n: usize) -> Result<(), io::Error> {
        skip::<8>(self, n)
    }
}

// Overengineering.jpg
fn skip<const BUF: usize>(mut rdr: impl Read, n: usize) -> Result<(), io::Error> {
    let mut buf = [0; BUF];
    for i in (0..n).step_by(BUF) {
        let b = std::cmp::min(BUF, n - i);
        rdr.read_exact(&mut buf[..b])?;
    }
    Ok(())
}
