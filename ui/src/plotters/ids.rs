use fds_toolbox_core::common::series::TimeSeriesView;
use ndarray::{Ix, Dimension, Ix0, Ix1, Ix2};


pub trait SeriesSource<Ix: Dimension> {
    fn iter_series(&self) -> Box<dyn Iterator<Item = TimeSeriesView<f32, Ix, f32>>>;
}

pub type SeriesSource0 = dyn SeriesSource<Ix0>;
pub type SeriesSource1 = dyn SeriesSource<Ix1>;
pub type SeriesSource2 = dyn SeriesSource<Ix2>;

pub trait IdSource {
    type Id;
    type Iter<'a>: Iterator<Item = Self::Id> + 'a
    where
        Self: 'a;
    fn iter_ids(&self) -> Self::Iter<'_>;
}

impl<IdSrc: IdSource> IdSource for &IdSrc {
    type Id = IdSrc::Id;
    type Iter<'a> = IdSrc::Iter<'a>
    where
        Self: 'a;
    fn iter_ids(&self) -> Self::Iter<'_> {
        (*self).iter_ids()
    }
}
