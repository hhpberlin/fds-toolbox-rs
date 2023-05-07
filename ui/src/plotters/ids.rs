use fds_toolbox_core::common::series::{TimeSeries0View, TimeSeries2Frame};

pub trait SeriesSourceLine {
    fn for_each_series(&self, f: &mut dyn for<'view> FnMut(TimeSeries0View<'view>));
}

pub trait SeriesSourceSlice {
    fn for_each_series(&self, f: &mut dyn for<'view> FnMut(TimeSeries2Frame<'view>));
}
