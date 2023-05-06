use fds_toolbox_core::common::series::{TimeSeriesView, TimeSeries0View, TimeSeries2Frame};
use ndarray::{Dimension, Ix1, Ix2, Ix3};

pub trait SeriesSource {
    type Item<'a>;
    fn for_each_series(&self, f: &mut dyn for<'a> FnMut(Self::Item<'a>));
}

pub type SeriesSourceLine<'a> = dyn for<'b> SeriesSource<Item<'b> = TimeSeries0View<'b>> + 'a;
pub type SeriesSourceSlice<'a> = dyn for<'b> SeriesSource<Item<'b> = TimeSeries2Frame<'b>> + 'a;