use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};

use fds_toolbox_core::common::series::TimeSeriesViewSource;
use plotters::{
    coord::{types::RangedCoordf32, ReverseCoordTranslate},
    prelude::{Cartesian2d, ChartContext, Circle, EmptyElement, PathElement, Text, CoordTranslate},
    series::{LineSeries, PointSeries},
    style::{Palette99, ShapeStyle, RED, Palette, Color},
};

use super::cartesian::{self, CartesianDrawer};

type PosF = (f32, f32);
type PosI = (i32, i32);

type Cartesian2df32 = Cartesian2d<RangedCoordf32, RangedCoordf32>;

#[derive(Debug)]
pub struct LinePlot<Id, DataSrc: TimeSeriesViewSource<Id>, IdSrc: IdSource<Id = Id>> {
    data_source: DataSrc,
    id_source: IdSrc,
}

impl<Id, DataSrc: TimeSeriesViewSource<Id>, IdSrc: IdSource<Id = Id>> LinePlot<Id, DataSrc, IdSrc> {
    pub fn new(data_source: DataSrc, id_source: IdSrc) -> Self {
        Self {
            data_source,
            id_source,
        }
    }
}
pub trait IdSource {
    type Id;
    type Iter<'a>: Iterator<Item = Self::Id> + 'a
    where
        Self: 'a;
    fn iter_ids(&self) -> Self::Iter<'_>;
}

impl<Id: Copy, DataSrc: TimeSeriesViewSource<Id>, IdSrc: IdSource<Id = Id>> CartesianDrawer
    for LinePlot<Id, DataSrc, IdSrc>
{
    fn draw<DB: plotters_iced::DrawingBackend>(
        &self,
        chart: &mut ChartContext<DB, Cartesian2df32>,
        state: &cartesian::State,
    ) {
        let hover_screen = state
            .hovered_point
            .map(|point| (point.x as i32, point.y as i32));

        let mut closest: Option<ClosestPoint<Id>> = None;

        // TODO: Avoid alloc by reusing iterator?
        let data = self
            .id_source
            .iter_ids()
            .filter_map(|id| self.data_source.get_time_series(id).map(|x| (id, x)));

        for (id, data) in data {
            // TODO: This could be better, but it works for now
            // This is used for assigning unique colors to each series
            let hash = {
                let mut hasher = DefaultHasher::new();
                data.values.stats.hash(&mut hasher);
                hasher.finish()
            };

            let color = Palette99::pick(hash as usize);

            // TODO: Extrapolate to the edges of the plot
            let data_iter = data.iter().filter(|(x, _y)| {
                state.x_range.contains(*x)
                // TODO: This would cause a flat line between the surrounding points to be drawn
                // && self.state.y_range.contains(*y)
            });

            // TODO: This allocs like crazy, one alloc for each point
            chart
                .draw_series(LineSeries::new(data_iter, color.stroke_width(2)))
                .expect("failed to draw chart data")
                .label(format!("{} ({})", data.name, data.unit))
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
                });

            if let Some(hover_screen) = hover_screen {
                closest = closest
                    .into_iter()
                    .chain(
                        data.iter()
                            .map(|x| ClosestPoint::new(id, x, hover_screen, chart.as_coord_spec())),
                    )
                    .fold(None, |a, b| match a {
                        None => Some(b),
                        Some(a) => Some(if a.distance_screen_sq < b.distance_screen_sq {
                            a
                        } else {
                            b
                        }),
                    });
            }
        }

        let hover = match hover_screen {
            Some(coord) => chart.as_coord_spec().reverse_translate(coord),
            None => None,
        };

        let hover = match closest {
            Some(ClosestPoint {
                point,
                distance_screen_sq: d,
                ..
            }) if d < 50.0_f32.powi(2) => Some(point),
            _ => hover,
        };

        // Draw cursor crosshair

        if let Some((x, y)) = hover {
            // cursor

            chart
                .draw_series(PointSeries::of_element(
                    hover.iter().copied(),
                    5,
                    ShapeStyle::from(&RED).filled(),
                    &|(x, y), size, style| {
                        EmptyElement::at((x, y))
                            + Circle::new((0, 0), size, style)
                            + Text::new(format!("{:?}", (x, y)), (0, 15), ("sans-serif", 15))
                    },
                ))
                .unwrap();

            // crosshair

            chart
                .draw_series(LineSeries::new(
                    [(state.x_range.min, y), (state.x_range.max, y)],
                    RED.stroke_width(1),
                ))
                .unwrap();

            chart
                .draw_series(LineSeries::new(
                    [(x, state.y_range.min), (x, state.y_range.max)],
                    RED.stroke_width(1),
                ))
                .unwrap();
        }
    }
}

struct ClosestPoint<Id> {
    id: Id,
    point: PosF,
    point_screen: PosI,
    distance_screen_sq: f32,
}

impl<Id> ClosestPoint<Id> {
    fn new(id: Id, point: PosF, hover_screen: PosI, coord_spec: &Cartesian2df32) -> Self {
        let point_screen = coord_spec.translate(&point);
        let distance_screen_sq = (point_screen.0 as f32 - hover_screen.0 as f32).powi(2)
            + (point_screen.1 as f32 - hover_screen.1 as f32).powi(2);

        Self {
            id,
            point,
            point_screen,
            distance_screen_sq,
        }
    }
}
