use std::{
    cell::RefCell,
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{Hash, Hasher},
};

use fds_toolbox_core::common::{range::RangeIncl, series::TimeSeriesViewSource};
use iced::{
    canvas::{Cache, Frame, Geometry},
    Command, Element, Length, Point, Size,
};
use plotters::{
    coord::{types::RangedCoordf32, ReverseCoordTranslate, Shift},
    prelude::*,
};
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Zoom { center: Position, factor: f32 },
    Hover { position: Point },
    Mouse { down: bool },
}

#[derive(Debug, Clone, Copy)]
pub enum Position {
    Screen(Point),
    Data((f32, f32)),
}

type Cartesian2df32 = Cartesian2d<RangedCoordf32, RangedCoordf32>;

pub struct Plot2DState {
    cache: Cache,
    x_range: RangeIncl<f32>,
    y_range: RangeIncl<f32>,
    hovered_point: Option<Point>,
    // Needs to be modified inside build_chart, which only receives a &self
    // chart_state: RefCell<Option<ChartState<Cart2D>>>,
    coord_spec: RefCell<Option<Cartesian2df32>>,
    mouse_down: bool,
}

impl Debug for Plot2DState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plot2D")
            .field("cache", &self.cache)
            .field("x_range", &self.x_range)
            .field("y_range", &self.y_range)
            .field("hovered_point", &self.hovered_point)
            // .field("coord_spec", &self.coord_spec)
            .field("mouse_down", &self.mouse_down)
            .finish()
    }
}

struct Plot2DInstance<'a, Id, Source: TimeSeriesViewSource<Id>, IdSrc: IdSource<Id = Id>> {
    state: &'a Plot2DState,
    ids: &'a IdSrc,
    source: &'a Source,
}

impl<'a, Id: Copy, Source: TimeSeriesViewSource<Id>, IdSrc: IdSource<Id = Id>> Chart<Message>
    for Plot2DInstance<'a, Id, Source, IdSrc>
{
    #[inline]
    fn draw<F: Fn(&mut Frame)>(&self, bounds: Size, draw_fn: F) -> Geometry {
        self.state.cache.draw(bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, _chart: ChartBuilder<DB>) {}

    fn draw_chart<DB: DrawingBackend>(&self, root: DrawingArea<DB, Shift>) {
        let mut chart = ChartBuilder::on(&root);
        let chart = chart.x_label_area_size(30).y_label_area_size(30).margin(20);

        // TODO: Avoid alloc by reusing iterator?
        let data = self
            .ids
            .iter_ids()
            .filter_map(|id| self.source.get_time_series(id).map(|x| (id, x)));

        let mut chart = chart
            .build_cartesian_2d(
                self.state.x_range.into_range(),
                self.state.y_range.into_range(),
            )
            .expect("failed to build chart");

        // Draws the grid and axis
        chart.configure_mesh().draw().expect("failed to draw mesh");

        let hover_screen = self
            .state
            .hovered_point
            .map(|point| (point.x as i32, point.y as i32));
        let mut closest: Option<ClosestPoint<Id>> = None;

        for (id, data) in data {
            // TODO: This could be better, but it works for now
            // This is used for assigning unique colors to each series
            let hash = {
                let mut hasher = DefaultHasher::new();
                data.values.stats.hash(&mut hasher);
                hasher.finish()
            };

            let color = Palette99::pick(hash as usize);

            chart
                .draw_series(LineSeries::new(
                    data.iter(),
                    color.stroke_width(2),
                ))
                .expect("failed to draw chart data")
                .label(format!("{} ({})", data.name, data.unit))
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2)))
                ;

            if let Some(hover_screen) = hover_screen {
                closest = closest
                    .into_iter()
                    .chain(
                        data.iter()
                            .map(|x| ClosestPoint::get(id, x, hover_screen, chart.as_coord_spec())),
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
                    [(self.state.x_range.min, y), (self.state.x_range.max, y)],
                    RED.stroke_width(1),
                ))
                .unwrap();

            chart
                .draw_series(LineSeries::new(
                    [(x, self.state.y_range.min), (x, self.state.y_range.max)],
                    RED.stroke_width(1),
                ))
                .unwrap();
        }

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()
            .expect("failed to draw chart labels");

        self.state
            .coord_spec
            .borrow_mut()
            .replace(chart.as_coord_spec().clone());
    }

    fn update(
        &mut self,
        event: iced::canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::canvas::Cursor,
    ) -> (iced::canvas::event::Status, Option<Message>) {
        let event = match event {
            iced::canvas::Event::Mouse(m) => m,
            iced::canvas::Event::Keyboard(_) => {
                return (iced::canvas::event::Status::Ignored, None)
            }
        };

        let p = match cursor.position_in(&bounds) {
            Some(p) => p,
            None => return (iced::canvas::event::Status::Ignored, None),
        };

        let message = match event {
            iced::mouse::Event::CursorEntered => Some(Message::Mouse { down: false }),
            iced::mouse::Event::CursorLeft => Some(Message::Mouse { down: false }),
            iced::mouse::Event::CursorMoved { position: _ } => Some(Message::Hover { position: p }),
            iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                Some(Message::Mouse { down: true })
            }
            iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                Some(Message::Mouse { down: false })
            }
            iced::mouse::Event::ButtonPressed(_) => None,
            iced::mouse::Event::ButtonReleased(_) => None,
            iced::mouse::Event::WheelScrolled { delta } => Some(Message::Zoom {
                // TODO: Actually calculate the center instead of this bullshit
                // center: self.chart_state.borrow().unwrap().,
                // center: (self.x_range.map(p.x), p.y),
                center: Position::Screen(p),
                factor: match delta {
                    // TODO: Treat line and pixel scroll differently
                    // TODO: Use a better zoom factor
                    // TODO: Look at x scroll
                    iced::mouse::ScrollDelta::Lines { y, .. } => (y / -10.0).exp(),
                    iced::mouse::ScrollDelta::Pixels { y, .. } => (y / -10.0).exp(),
                },
            }),
        };

        (iced::canvas::event::Status::Captured, message)
    }
}

struct ClosestPoint<Id> {
    id: Id,
    point: (f32, f32),
    point_screen: (i32, i32),
    distance_screen_sq: f32,
}

impl<Id> ClosestPoint<Id> {
    fn get(
        id: Id,
        point: (f32, f32),
        hover_screen: (i32, i32),
        coord_spec: &Cartesian2df32,
    ) -> Self {
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

impl Plot2DState {
    pub fn new() -> Self {
        Self {
            cache: Cache::new(),
            x_range: RangeIncl::new(0.0, 100.0),
            y_range: RangeIncl::new(0.0, 100.0),
            hovered_point: None,
            coord_spec: RefCell::new(None),
            mouse_down: false,
        }
    }

    pub fn zoom(&mut self, center: (f32, f32), factor: f32) {
        // dbg!(center);
        let (cx, cy) = center;
        self.x_range.zoom(cx, factor);
        self.y_range.zoom(cy, factor);
    }

    pub fn pan(&mut self, delta: (f32, f32)) {
        let (dx, dy) = delta;
        self.x_range.pan(dx);
        self.y_range.pan(dy);
    }

    fn map_pos_to_coord(&self, screen: Position) -> Option<(f32, f32)> {
        self.coord_spec
            .borrow()
            .as_ref()
            .and_then(|x| screen.into_data_coords(x))
    }

    fn map_screen_to_coord(&self, screen: Point) -> Option<(f32, f32)> {
        self.coord_spec
            .borrow()
            .as_ref()
            .and_then(|x| x.reverse_translate((screen.x as i32, screen.y as i32)))
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Zoom { center, factor } => {
                self.invalidate();
                let pos = self
                    .map_pos_to_coord(center)
                    .unwrap_or_else(|| (self.x_range.center(), self.y_range.center()));
                self.zoom(pos, factor);
            }
            Message::Hover { position } => {
                self.invalidate();
                let previous = self.hovered_point;
                self.hovered_point = Some(position);

                if self.mouse_down {
                    let previous = previous.and_then(|x| self.map_screen_to_coord(x));
                    let current = self.hovered_point.and_then(|x| self.map_screen_to_coord(x));

                    if let (Some(previous), Some(current)) = (previous, current) {
                        let delta = (previous.0 - current.0, previous.1 - current.1);
                        self.pan(delta);
                    }
                }
            }
            Message::Mouse { down } => {
                self.mouse_down = down;
            }
        }
        Command::none()
    }

    pub fn view<'a, Id: Copy + 'a, Source: TimeSeriesViewSource<Id>>(
        &'a self,
        source: &'a Source,
        ids: &'a (impl IdSource<Id = Id> + 'a),
    ) -> Element<'a, Message> {
        ChartWidget::new(Plot2DInstance {
            state: self,
            source,
            ids,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn invalidate(&mut self) {
        self.cache.clear();
    }
}

impl Default for Plot2DState {
    fn default() -> Self {
        Self::new()
    }
}

impl Position {
    pub fn into_data_coords(self, coord_spec: &Cartesian2df32) -> Option<(f32, f32)> {
        match self {
            Position::Data(p) => Some(p),
            Position::Screen(p) => coord_spec.reverse_translate((p.x as i32, p.y as i32)),
        }
    }
}

pub trait IdSource {
    type Id;
    // TODO: Remove dynamic dispatch when GATs are stable
    fn iter_ids(&self) -> Box<dyn Iterator<Item = Self::Id> + '_>;
}
