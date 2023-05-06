use std::{
    collections::hash_map::DefaultHasher,
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
};

use plotters::{
    coord::ReverseCoordTranslate,
    prelude::{ChartContext, Circle, CoordTranslate, EmptyElement, PathElement, Text},
    series::{LineSeries, PointSeries},
    style::{Color, Palette, Palette99, ShapeStyle, RED},
};

use super::{
    cartesian::{self, Cartesian2df32, CartesianDrawer},
    ids::SeriesSource0,
};

type PosF = (f32, f32);
type PosI = (i32, i32);

pub struct LinePlot {
    data_source: Box<SeriesSource0>,
}

impl Debug for LinePlot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinePlot").finish()
    }
}

impl LinePlot {
    pub fn new(data_source: Box<SeriesSource0>) -> Self {
        Self { data_source }
    }

    // pub fn new(data_source: impl SeriesSource) -> Self {
    //     Self {
    //         data_source: Box::new(data_source),
    //     }
    // }
}

impl CartesianDrawer for LinePlot {
    fn draw<DB: plotters_iced::DrawingBackend>(
        &self,
        chart: &mut ChartContext<DB, Cartesian2df32>,
        state: &cartesian::State,
    ) {
        let hover_screen = state
            .hovered_point
            .map(|point| (point.x as i32, point.y as i32));

        let mut closest: Option<ClosestPoint> = None;

        // TODO: Avoid alloc by reusing iterator?
        let data = self.data_source.iter_series();

        for view in data {
            // TODO: This could be better, but it works for now
            // This is used for assigning unique colors to each series
            let hash = {
                let mut hasher = DefaultHasher::new();
                view.values.stats.hash(&mut hasher);
                hasher.finish()
            };

            let color = Palette99::pick(hash as usize);

            // TODO: Extrapolate to the edges of the plot
            let data_iter = view.iter().filter(|(x, y)| {
                state.x_range.contains(*x) && state.y_range.contains(*y)
                // TODO: This would cause a flat line between the surrounding points to be drawn
                // && self.state.y_range.contains(*y)
            });

            // TODO: This allocs like crazy, one alloc for each point
            chart
                .draw_series(LineSeries::new(data_iter, color.stroke_width(2)))
                .expect("failed to draw chart data")
                .label(format!("{} ({})", view.name, view.unit))
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
                });

            if let Some(hover_screen) = hover_screen {
                closest = closest
                    .into_iter()
                    .chain(
                        view.iter()
                            .map(|x| ClosestPoint::new(x, hover_screen, chart.as_coord_spec())),
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
                // TODO: Fix this unwrap
                .unwrap();

            // crosshair

            chart
                .draw_series(LineSeries::new(
                    [(state.x_range.min, y), (state.x_range.max, y)],
                    RED.stroke_width(1),
                ))
                // TODO: Fix this unwrap
                .unwrap();

            chart
                .draw_series(LineSeries::new(
                    [(x, state.y_range.min), (x, state.y_range.max)],
                    RED.stroke_width(1),
                ))
                // TODO: Fix this unwrap
                .unwrap();
        }
    }
}

struct ClosestPoint {
    point: PosF,
    point_screen: PosI,
    distance_screen_sq: f32,
}

impl ClosestPoint {
    fn new(point: PosF, hover_screen: PosI, coord_spec: &Cartesian2df32) -> Self {
        let point_screen = coord_spec.translate(&point);
        let distance_screen_sq = (point_screen.0 as f32 - hover_screen.0 as f32).powi(2)
            + (point_screen.1 as f32 - hover_screen.1 as f32).powi(2);

        Self {
            point,
            point_screen,
            distance_screen_sq,
        }
    }
}
