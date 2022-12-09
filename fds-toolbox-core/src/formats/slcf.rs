use uom::si::f32::Time;

use crate::geom::Axis;

#[derive(Debug)]
pub struct Slice {
    files: Vec<SliceFile>,
    axis: Axis,
    cell_centered: bool,
}

#[derive(Debug)]
pub struct SliceFile {
    frames: Vec<SliceFrame>,
}

#[derive(Debug)]
pub struct SliceFrame {
    time: Time,
    data: ndarray::Array2<f32>,
}

// fn parse_slcf_2(rdr: impl Read) {
//     let buf = std::io::BufReader::new(rdr);
//     ()
// }

// struct EarlyEoF { missing_bytes: Option<u32> }

// fn parse_slcf(rdr: impl Read) -> Result<SliceFile, EarlyEoF> {
// }

// fn parse_str(rdr: impl Read) -> Result<String, EarlyEoF> {

// }

// fn parse_i32(rdr: impl Read) -> Result<i32, EarlyEoF> {
//     let mut buf = [0u8; 4];
//     rdr.read_exact(&mut buf).map_err(|x| { EarlyEoF { missing_bytes: None } })?;
//     Ok(i32::from_le_bytes(buf))
// }