use crate::common::series::Series2;
use crate::formats::fds::fds_slice_frame::FdsSliceFrame;
use crate::geom::bounds3int::{Bounds3I, Dimension3D};

pub struct FdsSlice {
    min_value: f32,
    max_value: f32,
    frames: Series2<FdsSliceFrame>,
    bounds: Bounds3I, 
    flat_dimension: Dimension3D,
    flat_dimension_position: i32,
    dimension_i: Dimension3D,
    dimension_j: Dimension3D,
    quantity: String,
    short_name: String,
    units: String,
}
