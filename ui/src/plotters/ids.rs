use fds_toolbox_core::common::series::TimeSeriesView;
use ndarray::{Dimension, Ix0, Ix1, Ix2, Ix3};

pub trait SeriesSource<Ix: Dimension> {
    //     fn iter_series(
    //         &self,
    //     ) -> Box<
    //         // dyn Iterator<Item = Box<dyn Viewable<Ix>>>,
    //         dyn Iterator<Item = TimeSeriesView<'_, f32, Ix, f32>>,
    //     >;

    fn for_each_series(&self, f: &mut dyn FnMut(TimeSeriesView<'_, f32, Ix, f32>));
}

pub type SeriesSource1<'a> = dyn SeriesSource<Ix1> + 'a;
pub type SeriesSource2<'a> = dyn SeriesSource<Ix2> + 'a;
pub type SeriesSource3<'a> = dyn SeriesSource<Ix3> + 'a;

/// This helps work with the lifetime of `TimeSeriesView`:
/// We want to borrow from a `TimeSeries`, but since the lifetime depends on the
/// lifetime of our `Arc` clone, we can't just return a reference.
/// This trait allows us to return a bundle of the `Arc` and the info needed to
/// borrow the view from it.
pub trait Viewable<Ix: Dimension> {
    fn view(&self) -> TimeSeriesView<'_, f32, Ix, f32>;
}

// impl<'a, Ix: Dimension> Viewable for TimeSeriesView<'a, f32, Ix, f32> {
//     type View = Self;
//     fn view(&self) -> Self::View {
//         self.clone()
//     }
// }

// impl<Ix: Dimension, T: Clone> Viewable<Ix> for T {
//     fn view(&self) -> TimeSeriesView<'_,f32, Ix, f32> {
//         self.clone()
//     }
// }
