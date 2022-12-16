use std::ops::Range;

use fds_toolbox_core::common::series::TimeSeriesViewSource;
use ndarray::{Axis, Dimension, Ix2};
use plotters::{prelude::Rectangle, style::HSLColor};

use super::{
    cartesian::{Cartesian2df32, CartesianDrawer},
    ids::IdSource,
};

pub struct Heatmap<Id, DataSrc: TimeSeriesViewSource<Id, f32, Ix2>, IdSrc: IdSource<Id = Id>> {
    data_source: DataSrc,
    id_source: IdSrc,
    frame: usize,
}

impl<Id: Copy, DataSrc: TimeSeriesViewSource<Id, f32, Ix2>, IdSrc: IdSource<Id = Id>>
    CartesianDrawer for Heatmap<Id, DataSrc, IdSrc>
{
    fn draw<DB: plotters_iced::DrawingBackend>(
        &self,
        chart: &mut plotters::prelude::ChartContext<DB, Cartesian2df32>,
        state: &super::cartesian::State,
    ) {
        // let plot_area = chart.plotting_area();

        let data = self
            .id_source
            .iter_ids()
            .filter_map(|id| self.data_source.get_time_series(id).map(|x| (id, x)));

        let t = self.frame;

        for (id, data) in data {
            // let Some(frame) = data.frame(t) else { continue; };

            // let w = frame.values.data.len_of(Axis(0));
            // let h = frame.values.data.len_of(Axis(1));

            // chart.draw_series(iter_2d(0..w, 0..h).map(|(x, y)| {
            //     let v = frame.values.data[[w, h]];

            //     Rectangle::new(
            //         [(x, y), (x + 1, y + 1)],
            //         HSLColor(
            //             240.0 / 360.0 - 240.0 / 360.0 * (v as f64 / 20.0),
            //             0.7,
            //             0.1 + 0.4 * v as f64 / 20.0,
            //         ),
            //     )
            // })
            // // .collect::<Vec<_>>()
            // );
        }
    }
}

fn iter_2d<X: Copy, Y>(x: Range<X>, y: Range<Y>) -> impl Iterator<Item = (X, Y)>
where
    Range<X>: IntoIterator<Item = X>,
    Range<Y>: IntoIterator<Item = Y> + Clone,
{
    x.into_iter()
        .flat_map(move |x| y.clone().into_iter().map(move |y| (x, y)))
}

impl<Id, DataSrc: TimeSeriesViewSource<Id, f32, Ix2>, IdSrc: IdSource<Id = Id>>
    Heatmap<Id, DataSrc, IdSrc>
{
    pub fn new(data_source: DataSrc, id_source: IdSrc) -> Self {
        Self {
            data_source,
            id_source,
            frame: 0,
        }
    }
}
