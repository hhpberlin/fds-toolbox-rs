use crate::common::series::Series2;
use crate::formats::fds::fds_slice_frame::FdsSliceFrame;
use crate::lazy_data::bounds3int;
use crate::lazy_data::bounds3int::{Bounds3Int, Dimension3D};

pub struct FdsSlice {
    min_value: f32,
    max_value: f32,
    frames: Series2<FdsSliceFrame>,
    bounds: Bounds3Int, 
    flat_dimension: Dimension3D,
    flat_dimension_position: i32,
    dimension_i: Dimension3D,
    dimension_j: Dimension3D,
    quantity: str,
    short_name: str,
    units: str
}
