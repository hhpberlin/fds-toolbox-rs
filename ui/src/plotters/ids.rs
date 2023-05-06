use fds_toolbox_core::common::series::TimeSeriesView;
use ndarray::{Ix, Dimension, Ix0, Ix1, Ix2};


pub trait SeriesSource<Ix: Dimension> {
    fn iter_series(&self) -> Box<dyn Iterator<Item = TimeSeriesView<f32, Ix, f32>>>;
}

pub type SeriesSource0 = dyn SeriesSource<Ix0>;
pub type SeriesSource1 = dyn SeriesSource<Ix1>;
pub type SeriesSource2 = dyn SeriesSource<Ix2>;

trait Viewable {
    type View;
    fn view(&self) -> Self::View;
}

// impl<'a, Ix: Dimension> Viewable for TimeSeriesView<'a, f32, Ix, f32> {
//     type View = Self;
//     fn view(&self) -> Self::View {
//         self.clone()
//     }
// }

impl<T: Clone> Viewable for T {
    type View = Self;
    fn view(&self) -> Self::View {
        self.clone()
    }
}
