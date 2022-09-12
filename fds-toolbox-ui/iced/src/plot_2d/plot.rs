use std::fmt::Debug;

use fds_toolbox_core::common::{range::Range, series::TimeSeriesViewSource};
use iced::{
    canvas::{Cache, Frame, Geometry},
    Element, Length, Size,
};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

#[derive(Debug, Clone, Copy)]
pub enum ChartMessage {
    Zoom { center: (f32, f32), factor: f32 },
}

#[derive(Debug)]
pub struct Plot2D<Id> {
    cache: Cache,
    ids: Vec<Id>,
    pub x_range: Range<f32>,
    pub y_range: Range<f32>,
}

struct Plot2DInstance<'a, Id, Source: TimeSeriesViewSource<Id>> {
    data: &'a Plot2D<Id>,
    source: &'a Source,
}

impl<Id: Copy, Source: TimeSeriesViewSource<Id>> Chart<ChartMessage>
    for Plot2DInstance<'_, Id, Source>
{
    #[inline]
    fn draw<F: Fn(&mut Frame)>(&self, bounds: Size, draw_fn: F) -> Geometry {
        self.data.cache.draw(bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, mut chart: ChartBuilder<DB>) {
        let chart = chart.x_label_area_size(30).y_label_area_size(30).margin(20);

        // TODO: Avoid alloc by reusing iterator?
        let data = self
            .data
            .ids
            .iter()
            .filter_map(|id| self.source.get_time_series(*id))
            .collect::<Vec<_>>();

        // let x_range = self.data.x_range.or_else(|| Range::from_iter_range(data.iter().map(|x| x.time_in_seconds.stats.range)));
        // let y_range = self.data.y_range.or_else(|| Range::from_iter_range(data.iter().map(|x| x.values.stats.range)));

        // let (x_range, y_range) = match (x_range, y_range) {
        //     (Some(x_range), Some(y_range)) => (x_range, y_range),
        //     // _ => return,
        //     _ => (Range::new(0.0, 1.0), Range::new(0.0, 1.0)),
        // };

        let x_range = self.data.x_range;
        let y_range = self.data.y_range;

        let mut chart = chart
            .build_cartesian_2d(x_range.into_range(), y_range.into_range())
            .expect("failed to build chart");

        chart.configure_mesh().draw().expect("failed to draw mesh");

        let color = Palette99::pick(4).mix(0.9);

        for data in data {
            chart
                .draw_series(LineSeries::new(data.iter(), color.stroke_width(2)))
                // TODO: Set labels
                // .label("y = x^2")
                // .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED))
                .expect("failed to draw chart data");
        }

        // TODO: Draw labels

        // chart
        //     .configure_series_labels()
        //     .background_style(&WHITE.mix(0.8))
        //     .border_style(&BLACK)
        //     .draw()
        //     .expect("failed to draw chart labels");
    }
}

impl<Id: Copy> Plot2D<Id> {
    pub fn new(data: Vec<Id>) -> Self {
        Self {
            cache: Cache::new(),
            ids: data,
            x_range: Range::new(0.0, 100.0),
            y_range: Range::new(0.0, 100.0),
        }
    }

    pub fn view<'a, Source: TimeSeriesViewSource<Id>>(
        &'a mut self,
        source: &'a Source,
    ) -> Element<'a, ChartMessage> {
        ChartWidget::new(Plot2DInstance { data: self, source })
            .width(Length::Fill)
            .height(Length::Fill)
            .on_mouse_event(Box::new(|e, p| {
                match e {
                    iced::mouse::Event::CursorEntered => None,
                    iced::mouse::Event::CursorLeft => None,
                    iced::mouse::Event::CursorMoved { position: _ } => None,
                    iced::mouse::Event::ButtonPressed(_) => None,
                    iced::mouse::Event::ButtonReleased(_) => None,
                    iced::mouse::Event::WheelScrolled { delta } => Some(ChartMessage::Zoom {
                        // TODO: Actually calculate the center instead of this bullshit
                        center: (p.x, p.y),
                        // center: (self.x_range.map(p.x), p.y),
                        factor: match delta {
                            // TODO: Treat line and pixel scroll differently
                            // TODO: Use a better zoom factor
                            // TODO: Look at x scroll
                            iced::mouse::ScrollDelta::Lines { y, .. } => (y / 50.0).exp(),
                            iced::mouse::ScrollDelta::Pixels { y, .. } => (y / 50.0).exp(),
                        },
                    }),
                }
            }))
            .into()
    }

    pub fn zoom(&mut self, center: (f32, f32), factor: f32) {
        self.cache.clear();
        let (cx, cy) = center;
        self.x_range.zoom(cx, factor);
        self.y_range.zoom(cy, factor);
    }
}

impl<Id: Copy> Default for Plot2D<Id> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
