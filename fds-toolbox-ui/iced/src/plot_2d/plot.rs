use std::{fmt::Debug, cell::RefCell};

use fds_toolbox_core::common::{range::Range, series::TimeSeriesViewSource};
use iced::{
    canvas::{Cache, Frame, Geometry},
    Element, Length, Size, Point, Command,
};
use plotters::{prelude::*, coord::{ReverseCoordTranslate, types::RangedCoordf32}, chart::ChartState};
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

#[derive(Debug, Clone, Copy)]
pub enum ChartMessage {
    Zoom { center: Point, factor: f32 },
    Hover { position: Point },
}

type Cart2D = Cartesian2d<RangedCoordf32, RangedCoordf32>;

// #[derive(Debug)]
pub struct Plot2D<Id> {
    cache: Cache,
    ids: Vec<Id>,
    x_range: Range<f32>,
    y_range: Range<f32>,
    hovered_point: Option<(f32, f32)>,
    // Needs to be modified inside build_chart, which only receives a &self
    chart_state: RefCell<Option<ChartState<Cart2D>>>,
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

        //TODO
        // TODO: Avoid alloc by reusing iterator?
        let data = self
            .data
            .ids
            .iter()
            .filter_map(|id| self.source.get_time_series(*id));

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

        let hover = match self.data.hovered_point {
            Some((x, y)) => chart.as_coord_spec().reverse_translate((x as i32, y as i32)),
            _ => None,
        };

        if let Some((x, y)) = hover {
            // let (chart_x_range, chart_y_range) = chart.plotting_area().get_pixel_range();
            
            chart.draw_series(PointSeries::of_element(
                hover.iter().copied(),
                5,
                ShapeStyle::from(&RED).filled(),
                &|(x, y), size, style| {
                    EmptyElement::at((x, y))
                        + Circle::new((0, 0), size, style)
                        // MAX/2 to avoid overflow
                        // TODO: Find a better way to do this
                        // + PathElement::new([(chart_x_range.start - x as i32, 0), (chart_x_range.end - x as i32, 0)], style.clone())
                        // + PathElement::new([(0, i32::MIN/2), (0, i32::MAX/2)], style.clone())
                        + Text::new(format!("{:?}", (x, y)), (0, 15), ("sans-serif", 15))
                },
            )).unwrap();
    
            chart.draw_series(LineSeries::new([(x_range.min, y), (x_range.max, y)], RED.stroke_width(1))).unwrap();
            chart.draw_series(LineSeries::new([(x, y_range.min), (x, y_range.max)], RED.stroke_width(1))).unwrap();
        }
        // TODO: Draw labels

        // chart
        //     .configure_series_labels()
        //     .background_style(&WHITE.mix(0.8))
        //     .border_style(&BLACK)
        //     .draw()
        //     .expect("failed to draw chart labels");

        self.data.chart_state.borrow_mut().replace(chart.into_chart_state());
    }
}

impl<Id: Copy> Plot2D<Id> {
    pub fn new(data: Vec<Id>) -> Self {
        Self {
            cache: Cache::new(),
            ids: data,
            x_range: Range::new(0.0, 100.0),
            y_range: Range::new(0.0, 100.0),
            hovered_point: None,
            chart_state: RefCell::new(None),
        }
    }

    pub fn update(&mut self, message: ChartMessage) -> Command<ChartMessage> {
        match message {
            ChartMessage::Zoom { center, factor } => {
                self.zoom((center.x, center.y), factor);
                // dbg!(self.chart.x_range);
                // dbg!(self.chart.y_range);
            }
            ChartMessage::Hover { position } => self.hovered_point = Some((position.x, position.y)),
        }
        self.invalidate();
        Command::none()
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
                    iced::mouse::Event::CursorMoved { position: _ } => { Some(ChartMessage::Hover { position: p }) },
                    iced::mouse::Event::ButtonPressed(_) => None,
                    iced::mouse::Event::ButtonReleased(_) => None,
                    iced::mouse::Event::WheelScrolled { delta } => Some(ChartMessage::Zoom {
                        // TODO: Actually calculate the center instead of this bullshit
                        center: p,
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
        dbg!(center);
        let (cx, cy) = center;
        self.x_range.zoom(cx, factor);
        self.y_range.zoom(cy, factor);
    }

    pub fn invalidate(&mut self) {
        self.cache.clear();
    }
}

impl<Id: Copy> Default for Plot2D<Id> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
