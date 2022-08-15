use uom::si::f32::Time;


pub struct SliceFile {
    frames: Vec<SliceFrame>,
}

pub struct SliceFrame {
    time: Time,
    data: ndarray::Array2<f32>,

}