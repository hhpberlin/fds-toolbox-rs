use std::{io::{self, Read}};

use byteorder::{ReadBytesExt, LittleEndian};
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
    fn read_vlq_u32(&mut self) -> Result<u32, io::Error>;
    fn skip(&mut self, n: usize) -> Result<(), io::Error>;
}

impl<T: Read> ReadExt for T {
    fn read_string(&mut self) -> Result<String, ReadStrErr> {
        // let len = self.read_vlq_u32()?;
        let len = self.read_u32::<LittleEndian>()?;
        let len = len as usize;

        let mut buf: Vec<u8> = vec![0u8; len];
        self.read_exact(&mut buf).map_err(|err| ReadStrErr::IoBuf(err, len))?;
        
        let from_utf8 = String::from_utf8(buf)?;
        dbg!(&from_utf8);
        Ok(from_utf8)
    }

    fn read_vlq_u32(&mut self) -> Result<u32, io::Error> {
        // TODO: Report overflows
        let mut result = 0;
        let mut shift = 0;
        loop {
            let b = self.read_u8()?;
            result |= ((b & !(1u8 << 7)) as u32) << shift;
            if b & (1u8 << 7) == 0 {
                break;
            }
            shift += 7;
        }
        Ok(result)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn as_reader(bytes: &[u8]) -> impl Read + '_ {
        bytes
    }

    #[test]
    fn skip() {
        let mut rdr = as_reader(&[1, 2, 3]);
        assert_eq!(rdr.read_u8().unwrap(), 1);
        rdr.skip(1).expect("Failed to skip");
        assert_eq!(rdr.read_u8().unwrap(), 3);
    }

    #[test]
    fn read_vlq_u32_0() {
        let mut rdr = as_reader(&[0b00000000]);
        assert_eq!(rdr.read_vlq_u32().unwrap(), 0);
    }

    #[test]
    fn read_vlq_u32_one_block() {
        let mut rdr = as_reader(&[0b01000001]);
        assert_eq!(rdr.read_vlq_u32().unwrap(), 65);
    }

    #[test]
    fn read_vlq_u32_two_blocks() {
        let mut rdr = as_reader(&[0b10000010, 0b00000001]);
        assert_eq!(rdr.read_vlq_u32().unwrap(), 130);
    }
}