use std::{cell::RefCell, fmt::Debug};

use fds_toolbox_core::common::range::RangeIncl;
use iced::mouse::Cursor;
use iced::widget::canvas::Event;
use iced::{
    event::Status,
    mouse,
    widget::canvas::{Cache, Frame, Geometry},
    Element, Length, Point, Size,
};
use plotters::{
    coord::{types::RangedCoordf32, ReverseCoordTranslate, Shift},
    prelude::*,
};
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

type PosF = (f32, f32);
type PosI = (i32, i32);

pub type Cartesian2df32 = Cartesian2d<RangedCoordf32, RangedCoordf32>;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Zoom { center: Position, factor: f32 },
    Hover { position: Point },
    MousePress { down: bool },
    Invalidate,
}

#[derive(Debug, Clone, Copy)]
pub enum Position {
    Screen(Point),
    Data(PosF),
}

pub struct State {
    // Plot coordinates
    pub x_range: RangeIncl<f32>,
    pub y_range: RangeIncl<f32>,
    // Screen coordinates
    pub hovered_point: Option<Point>,
    // Needs to be modified inside build_chart, which only receives a &self
    pub coord_spec: RefCell<Option<Cartesian2df32>>,
    pub mouse_down: bool,
    cache: Cache,
}

impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plot2D")
            .field("x_range", &self.x_range)
            .field("y_range", &self.y_range)
            .field("hovered_point", &self.hovered_point)
            // .field("coord_spec", &self.coord_spec)
            .field("mouse_down", &self.mouse_down)
            .field("cache", &self.cache)
            .finish()
    }
}

#[derive(Debug)]
pub struct CartesianPlot<'a, Drawer: CartesianDrawer + 'a> {
    drawer: Drawer,
    state: &'a RefCell<State>,
}

pub struct Crosshair<'a> {
    position: PosF,
    label: &'a str,
}

pub trait CartesianDrawer {
    fn draw<DB: DrawingBackend>(
        &self,
        chart: &mut ChartContext<DB, Cartesian2d<RangedCoordf32, RangedCoordf32>>,
        state: &State,
    );
}

impl<'a, Drawer: CartesianDrawer + 'a> CartesianPlot<'a, Drawer> {
    pub fn new(drawer: Drawer, state: &'a RefCell<State>) -> Self {
        Self { drawer, state }
    }
}

impl<'a, Drawer: CartesianDrawer + 'a> Chart<Message> for CartesianPlot<'a, Drawer> {
    // TODO: See if I can make use of this
    //       Currently not used because this persists between tabs
    type State = ();

    #[inline]
    fn draw<F: Fn(&mut Frame)>(&self, bounds: Size, draw_fn: F) -> Geometry {
        // TODO: This might panic
        self.state.borrow().cache.draw(bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, _chart: ChartBuilder<DB>) {}

    fn draw_chart<DB: DrawingBackend>(&self, _state: &Self::State, root: DrawingArea<DB, Shift>) {
        let mut chart = ChartBuilder::on(&root);
        let chart = chart.x_label_area_size(30).y_label_area_size(30).margin(20);

        // TODO: This might panic
        let state = self.state.borrow();

        let mut chart = chart
            .build_cartesian_2d(state.x_range.into_range(), state.y_range.into_range())
            .expect("failed to build chart");

        // Draws the grid and axis
        chart.configure_mesh().draw().expect("failed to draw mesh");

        state
            .coord_spec
            // TODO: This might panic
            .borrow_mut()
            .replace(chart.as_coord_spec().clone());

        self.drawer.draw(&mut chart, &state);

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .expect("failed to draw chart labels");
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        bounds: iced::Rectangle,
        cursor: Cursor,
    ) -> (Status, Option<Message>) {
        // TODO: Handle touch events

        let event = match event {
            Event::Mouse(m) => m,
            // TODO: Support touch events
            Event::Touch(_) | Event::Keyboard(_) => return (Status::Ignored, None),
        };

        let p = match cursor.position_in(&bounds) {
            Some(p) => p,
            None => return (Status::Ignored, None),
        };

        let message = match event {
            mouse::Event::CursorEntered => Some(Message::MousePress { down: false }),
            mouse::Event::CursorLeft => Some(Message::MousePress { down: false }),
            mouse::Event::CursorMoved { position: _ } => Some(Message::Hover { position: p }),
            mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                Some(Message::MousePress { down: true })
            }
            mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                Some(Message::MousePress { down: false })
            }
            mouse::Event::ButtonPressed(_) => None,
            mouse::Event::ButtonReleased(_) => None,
            mouse::Event::WheelScrolled { delta } => Some(Message::Zoom {
                center: Position::Screen(p),
                factor: match delta {
                    // TODO: Treat line and pixel scroll differently
                    // TODO: Use a better zoom factor
                    // TODO: Look at x scroll
                    mouse::ScrollDelta::Lines { y, .. } => (y / -10.0).exp(),
                    mouse::ScrollDelta::Pixels { y, .. } => (y / -10.0).exp(),
                },
            }),
        };

        if let Some(message) = message {
            self.state.borrow_mut().update(message);
        }

        (Status::Captured, message)
    }
}

impl State {
    pub fn new(x_range: RangeIncl<f32>, y_range: RangeIncl<f32>) -> Self {
        Self {
            x_range,
            y_range,
            hovered_point: None,
            coord_spec: RefCell::new(None),
            mouse_down: false,
            cache: Cache::new(),
        }
    }

    pub fn zoom(&mut self, center: PosF, factor: f32) {
        // dbg!(center);
        let (cx, cy) = center;
        self.x_range.zoom(cx, factor);
        self.y_range.zoom(cy, factor);
    }

    pub fn pan(&mut self, delta: PosF) {
        let (dx, dy) = delta;
        self.x_range.pan(dx);
        self.y_range.pan(dy);
    }

    fn map_pos_to_coord(&self, screen: Position) -> Option<PosF> {
        self.coord_spec
            .borrow_mut()
            .as_ref()
            .and_then(|x| screen.into_data_coords(x))
    }

    fn map_screen_to_coord(&self, screen: Point) -> Option<PosF> {
        self.coord_spec
            .borrow_mut()
            .as_ref()
            .and_then(|x| x.reverse_translate((screen.x as i32, screen.y as i32)))
    }

    pub fn update(&mut self, message: Message) {
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
            Message::MousePress { down } => {
                self.mouse_down = down;
            }
            Message::Invalidate => {
                self.invalidate();
            }
        }
    }

    pub fn invalidate(&mut self) {
        self.cache.clear();
    }
}

pub fn cartesian<'a>(
    drawer: impl CartesianDrawer + 'a,
    state: &'a RefCell<State>,
) -> Element<'a, Message> {
    ChartWidget::new(CartesianPlot { drawer, state })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// impl Default for State {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl Position {
    pub fn into_data_coords(self, coord_spec: &Cartesian2df32) -> Option<PosF> {
        match self {
            Position::Data(p) => Some(p),
            Position::Screen(p) => coord_spec.reverse_translate((p.x as i32, p.y as i32)),
        }
    }
}
