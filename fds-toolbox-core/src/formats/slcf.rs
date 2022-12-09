use ndarray::Axis;
use uom::si::f32::Time;

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

// impl TimeSeriesViewSource<Glo, f32, Ix2> for Simulations {
//     fn view(&self, id: Id) -> Option<TimeSeriesView<f32, Ix2>> {
//         let slice = self.get_slice(id)?;
//         let frames = slice.frames.iter().map(|frame| {
//             let time = frame.time;
//             let data = frame.data.clone();
//             TimeSeriesFrame { time, data }
//         }).collect();
//         Some(TimeSeriesView { frames })
//     }
// }