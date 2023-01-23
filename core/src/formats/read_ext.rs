use std::{
    fmt::Display,
    io::{self, Read},
    num::TryFromIntError,
    ops::{Bound, Range, RangeBounds},
};

use byteorder::{LittleEndian, ReadBytesExt};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReadStrErr {
    #[error("Problem reading block: {0}")]
    Block(#[from] ReadBlockErr),
    #[error("UTF-8 parsing error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

#[derive(Error, Debug)]
pub enum ReadBlockErr {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("IO error (expected to read {1} bytes): {0}")]
    IoBuf(io::Error, usize),
    #[error("Couldn't convert size of {0} bytes to native pointer size: {1}")]
    InvalidLength(u32, TryFromIntError),
    #[error("Mismatched postfix length: {0} bytes at start != {1} bytes at end")]
    MismatchedPostfixLength(usize, usize),
    #[error("Length out of range: {0} not in {1:?}..{2:?}")]
    SizeOutOfRange(usize, Bound<usize>, Bound<usize>),
}

pub trait U32Ext {
    fn try_into_usize(self) -> Result<usize, ReadBlockErr>;
}

impl U32Ext for u32 {
    fn try_into_usize(self) -> Result<usize, ReadBlockErr> {
        usize::try_from(self).map_err(|e| ReadBlockErr::InvalidLength(self, e))
    }
}

#[derive(Error, Debug)]
pub enum ReadValErr<T: Display> {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Expected {0}, got {1}")]
    WrongVal(T, T),
}

pub trait ReadExt {
    // Convenience functions
    fn read_fortran_string(&mut self) -> Result<String, ReadStrErr> {
        self.read_fortran_string_bounded_option::<Range<usize>>(None)
    }
    fn read_fortran_string_bounded<R: RangeBounds<usize>>(
        &mut self,
        size_range: R,
    ) -> Result<String, ReadStrErr> {
        self.read_fortran_string_bounded_option(Some(size_range))
    }
    // to call this
    fn read_fortran_string_bounded_option<R: RangeBounds<usize>>(
        &mut self,
        size_range: Option<R>,
    ) -> Result<String, ReadStrErr>;

    // Convenience functions
    fn read_fortran_block(&mut self) -> Result<Vec<u8>, ReadBlockErr> {
        self.read_fortran_block_bounded_option::<Range<usize>>(None)
    }
    fn read_fortran_block_bounded<R: RangeBounds<usize>>(
        &mut self,
        size_range: R,
    ) -> Result<Vec<u8>, ReadBlockErr> {
        self.read_fortran_block_bounded_option(Some(size_range))
    }
    // to call this
    fn read_fortran_block_bounded_option<R: RangeBounds<usize>>(
        &mut self,
        size_range: Option<R>,
    ) -> Result<Vec<u8>, ReadBlockErr>;

    fn read_fixed_u32(&mut self, num: u32) -> Result<(), ReadValErr<u32>>;

    fn skip(&mut self, n: usize) -> Result<(), io::Error>;
}

impl<T: Read> ReadExt for T {
    fn read_fortran_string_bounded_option<R: RangeBounds<usize>>(
        &mut self,
        size_range: Option<R>,
    ) -> Result<String, ReadStrErr> {
        let block = self.read_fortran_block_bounded_option(size_range)?;
        Ok(String::from_utf8(block)?)
    }

    fn read_fortran_block_bounded_option<R: RangeBounds<usize>>(
        &mut self,
        size_range: Option<R>,
    ) -> Result<Vec<u8>, ReadBlockErr> {
        let len = self.read_u32::<LittleEndian>()?;
        let prefix_len = usize::try_from(len).map_err(|x| ReadBlockErr::InvalidLength(len, x))?;

        if let Some(range) = size_range {
            if !range.contains(&prefix_len) {
                return Err(ReadBlockErr::SizeOutOfRange(
                    prefix_len,
                    range.start_bound().cloned(),
                    range.end_bound().cloned(),
                ));
            }
        }

        let mut buf: Vec<u8> = vec![0u8; prefix_len];
        self.read_exact(&mut buf)
            .map_err(|err| ReadBlockErr::IoBuf(err, prefix_len))?;

        let postfix_len = self.read_u32::<LittleEndian>()?;
        let postfix_len = usize::try_from(postfix_len)
            .map_err(|x| ReadBlockErr::InvalidLength(postfix_len, x))?;

        if postfix_len != prefix_len {
            return Err(ReadBlockErr::MismatchedPostfixLength(
                prefix_len,
                postfix_len,
            ));
        }

        Ok(buf)
    }

    // fn read_vlq_u32(&mut self) -> Result<u32, io::Error> {
    //     // TODO: Report overflows
    //     let mut result = 0;
    //     let mut shift = 0;
    //     loop {
    //         let b = self.read_u8()?;
    //         result |= ((b & !(1u8 << 7)) as u32) << shift;
    //         if b & (1u8 << 7) == 0 {
    //             break;
    //         }
    //         shift += 7;
    //     }
    //     Ok(result)
    // }

    fn read_fixed_u32(&mut self, num: u32) -> Result<(), ReadValErr<u32>> {
        match self.read_u32::<LittleEndian>() {
            Ok(x) if x == num => Ok(()),
            Ok(x) => Err(ReadValErr::WrongVal(num, x)),
            Err(err) => Err(ReadValErr::Io(err)),
        }
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
    // use std::assert_matches::assert_matches;

    use super::*;

    fn as_reader(bytes: &[u8]) -> impl Read + '_ {
        bytes
    }

    fn slice_u32_to_u8(bytes: &[u32]) -> Vec<u8> {
        bytes
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>()
        // Endianness-dependent
        // unsafe { std::mem::transmute::<&[u32], &[u8]>(bytes) }
    }

    #[test]
    fn skip() {
        let mut rdr = as_reader(&[1, 2, 3]);
        assert_eq!(rdr.read_u8().unwrap(), 1);
        rdr.skip(1).expect("Failed to skip");
        assert_eq!(rdr.read_u8().unwrap(), 3);
    }

    // #[test]
    // fn read_vlq_u32_0() {
    //     let mut rdr = as_reader(&[0b00000000]);
    //     assert_eq!(rdr.read_vlq_u32().unwrap(), 0);
    // }

    // #[test]
    // fn read_vlq_u32_one_block() {
    //     let mut rdr = as_reader(&[0b01000001]);
    //     assert_eq!(rdr.read_vlq_u32().unwrap(), 65);
    // }

    // #[test]
    // fn read_vlq_u32_two_blocks() {
    //     let mut rdr = as_reader(&[0b10000010, 0b00000001]);
    //     assert_eq!(rdr.read_vlq_u32().unwrap(), 130);
    // }

    #[test]
    fn read_fortran_block() {
        let data = slice_u32_to_u8(&[3 * 4, 1, 2, 3, 3 * 4]);
        let mut rdr = as_reader(&data);

        let block = rdr.read_fortran_block_bounded(3 * 4..=3 * 4).unwrap();
        assert_eq!(block, slice_u32_to_u8(&[1, 2, 3]));
    }

    #[test]
    fn read_fortran_block_length_oustide_of_range() {
        let data = slice_u32_to_u8(&[3 * 4, 0, 1, 2, 3 * 4]);
        let mut rdr = as_reader(&data);

        let block = rdr.read_fortran_block_bounded(4 * 4..=4 * 4);

        // TODO: Use assert_matches once it's stable

        match block {
            Err(ReadBlockErr::SizeOutOfRange(12, Bound::Included(16), Bound::Included(16))) => (),
            x => panic!("Unexpected return: {:?}", x),
        }
    }

    #[test]
    fn read_fortran_block_wrong_postfix() {
        let data = slice_u32_to_u8(&[3 * 4, 0, 1, 2, 4 * 4]);
        let mut rdr = as_reader(&data);

        let block = rdr.read_fortran_block_bounded(3 * 4..=3 * 4);

        match block {
            Err(ReadBlockErr::MismatchedPostfixLength(12, 16)) => (),
            x => panic!("Unexpected return: {:?}", x),
        }
    }

    #[test]
    fn read_magic_num() {
        let data = slice_u32_to_u8(&[0x12345678]);
        let mut rdr = as_reader(&data);

        rdr.read_fixed_u32(0x12345678).unwrap();
    }

    #[test]
    fn read_wrong_magic_num() {
        let data = slice_u32_to_u8(&[0x12345678]);
        let mut rdr = as_reader(&data);

        let res = rdr.read_fixed_u32(0x12345679);

        match res {
            Err(ReadValErr::WrongVal(0x12345679, 0x12345678)) => (),
            x => panic!("Unexpected return: {:?}", x),
        }
    }
}
