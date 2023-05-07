use fds_toolbox_core::common::series::{TimeSeriesView, TimeSeries0View, TimeSeries2Frame};
use ndarray::{Dimension, Ix1, Ix2, Ix3};

pub trait SeriesSourceLine {
    fn for_each_series(&self, f: &mut dyn for<'view> FnMut(TimeSeries0View<'view>));
}

pub trait SeriesSourceSlice {
    fn for_each_series(&self, f: &mut dyn for<'view> FnMut(TimeSeries2Frame<'view>));
}
