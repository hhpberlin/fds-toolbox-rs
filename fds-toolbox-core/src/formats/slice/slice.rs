use crate::common::series::Series2;
use crate::formats::slice::slice_frame::SliceFrame;
use crate::geom::bounds3int::{Bounds3I, Dimension3D};

pub struct Slice {
    min_value: f32,
    max_value: f32,
    frames: Series2,
    bounds: Bounds3I,
    flat_dimension: Dimension3D,
    flat_dimension_position: i32,
    dimension_i: Dimension3D,
    dimension_j: Dimension3D,
    quantity: String,
    short_name: String,
    units: String,
}

impl Slice {
    fn new(reader: impl Reder) -> Slice
    {
        Slice { min_value: (), max_value: (), frames: (), bounds: (), flat_dimension: (), flat_dimension_position: (), dimension_i: (), dimension_j: (), quantity: (), short_name: (), units: () }
    }
}
