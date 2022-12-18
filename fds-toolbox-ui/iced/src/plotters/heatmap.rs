use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use fds_toolbox_core::common::series::TimeSeriesViewSource;
use ndarray::{Axis, Ix3};
use plotters::{
    prelude::Rectangle,
    style::{Color, HSLColor, Palette, Palette99},
};

use super::{
    cartesian::{Cartesian2df32, CartesianDrawer},
    ids::IdSource,
};

pub struct Heatmap<Id, DataSrc: TimeSeriesViewSource<Id, f32, Ix3>, IdSrc: IdSource<Id = Id>> {
    data_source: DataSrc,
    id_source: IdSrc,
    frame: usize,
}

impl<Id: Copy, DataSrc: TimeSeriesViewSource<Id, f32, Ix3>, IdSrc: IdSource<Id = Id>>
    CartesianDrawer for Heatmap<Id, DataSrc, IdSrc>
{
    fn draw<DB: plotters_iced::DrawingBackend>(
        &self,
        chart: &mut plotters::prelude::ChartContext<DB, Cartesian2df32>,
        _state: &super::cartesian::State,
    ) {
        let data = self.id_source.iter_ids();
        // .filter_map(|id| self.data_source.get_time_series(id).map(|x| (id, x)));

        for id in data {
            let data = self.data_source.get_time_series(id);
            let Some(data) = data else { continue; };

            let hash = {
                let mut hasher = DefaultHasher::new();
                data.values.stats.hash(&mut hasher);
                hasher.finish()
            };

            let _color = Palette99::pick(hash as usize);

            let t = self.frame;

            let Some(frame) = data.view_frame(t) else { continue; };

            let w = frame.values.data.len_of(Axis(0));
            let h = frame.values.data.len_of(Axis(1));

            chart
                .draw_series(
                    iter_2d(0..w, 0..h)
                        .map(|(x, y)| {
                            let v = frame.values.data[[x, y]];
                            let x = x as f32;
                            let y = y as f32;

                            Rectangle::new(
                                [(x, y), (x + 1., y + 1.)],
                                HSLColor(
                                    240.0 / 360.0 - 240.0 / 360.0 * (v as f64 / 20.0),
                                    0.7,
                                    0.1 + 0.4 * v as f64 / 20.0,
                                )
                                .filled(),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
                // TODO: Fix this unwrap
                .unwrap();
        }
    }
}

fn iter_2d<X: Copy, Y>(
    x: impl IntoIterator<Item = X>,
    y: impl IntoIterator<Item = Y> + Clone,
) -> impl Iterator<Item = (X, Y)> {
    x.into_iter()
        .flat_map(move |x| y.clone().into_iter().map(move |y| (x, y)))
}

impl<Id, DataSrc: TimeSeriesViewSource<Id, f32, Ix3>, IdSrc: IdSource<Id = Id>>
    Heatmap<Id, DataSrc, IdSrc>
{
    pub fn new(data_source: DataSrc, id_source: IdSrc, frame: usize) -> Self {
        Self {
            data_source,
            id_source,
            frame,
        }
    }
}
