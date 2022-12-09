use uom::si::f32::Time;

pub struct FdsSliceFrame {
    time: Time,
    values: Vec<Vec<f32>>,
    min_value: f32,
    max_value: f32,
}

// impl FdsSliceFrame {
//     fn new(reader: &mut impl Read, slice: FdsSlice, block: i32) -> Result<FdsSliceFrame, &str> {
//         let mut ret: FdsSliceFrame = FdsSliceFrame {
//             time: Time::new::<second>(reader.read_f32()),
//             values: vec![],
//             min_value: f32::MAX,
//             max_value: f32::MIN,
//         };
//         let _ = reader.skip(1);

//         let block_size = reader.read_i32();
//         match block_size {
//             None => return Err("What is dis"),
//             Some(blk) => {
//                 if block * 4 != blk {
//                     return Err("bad block");
//                 }
//             }
//         }
//         Ok(ret)
//     }
// }
