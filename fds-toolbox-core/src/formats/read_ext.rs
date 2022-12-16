use std::io::{self, Read};

use byteorder::ReadBytesExt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReadStrErr {
    #[error("IO error")]
    Io(io::Error),
    #[error("UTF-8 error")]
    Utf8(std::string::FromUtf8Error),
}

pub trait ReadExt {
    #[must_use]
    fn read_string(&mut self) -> Result<String, ReadStrErr>;
    #[must_use]
    fn skip(&mut self, n: usize) -> Result<(), io::Error>;
}

impl<T: Read> ReadExt for T {
    fn read_string(&mut self) -> Result<String, ReadStrErr> {
        let len = self
            .read_u32::<byteorder::BigEndian>()
            .map_err(ReadStrErr::Io)?;
        let mut buf: Vec<u8> = vec![0u8; len as usize];
        self.read_exact(&mut buf).map_err(ReadStrErr::Io)?;
        String::from_utf8(buf).map_err(ReadStrErr::Utf8)
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
