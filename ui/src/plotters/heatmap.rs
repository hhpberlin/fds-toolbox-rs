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
    ids::{IdSource, SeriesSource2},
};

pub struct Heatmap {
    data_source: Box<SeriesSource2>,
}

impl CartesianDrawer for Heatmap
{
    fn draw<DB: plotters_iced::DrawingBackend>(
        &self,
        chart: &mut plotters::prelude::ChartContext<DB, Cartesian2df32>,
        _state: &super::cartesian::State,
    ) {
        let data = self.data_source.iter_series();

        for view in data {
            let hash = {
                let mut hasher = DefaultHasher::new();
                view.values.stats.hash(&mut hasher);
                hasher.finish()
            };

            let _color = Palette99::pick(hash as usize);

            let w = view.values.data.len_of(Axis(0));
            let h = view.values.data.len_of(Axis(1));

            chart
                .draw_series(
                    iter_2d(0..w, 0..h)
                        .map(|(x, y)| {
                            let v = view.values.data[[x, y]];
                            let x = x as f32;
                            let y = y as f32;

                            Rectangle::new(
                                [(x, y), (x + 1., y + 1.)],
                                HSLColor(
                                    // 240.0 / 360.0 - 240.0 / 360.0 * (v as f64 / 20.0),
                                    v as f64 * 2000.0,
                                    0.7,
                                    0.1 + 0.4 * v as f64 / 20.0,
                                )
                                .filled(),
                            )
                        })
                        // .collect::<Vec<_>>(),
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

impl Heatmap
{
    pub fn new(data_source: Box<SeriesSource2>) -> Self {
        Self {
            data_source,
        }
    }
}
