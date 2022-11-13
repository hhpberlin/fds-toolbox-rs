use uom::si::f32::Time;

#[derive(Debug)]
pub struct SliceFile {
    frames: Vec<SliceFrame>,
}

#[derive(Debug)]
pub struct SliceFrame {
    time: Time,
    data: ndarray::Array2<f32>,
}
