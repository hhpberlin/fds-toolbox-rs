use druid::{Data, Widget};
use fds_toolbox_core::common::range::RangeIncl;
use plotters::{prelude::Cartesian2d, coord::{types::RangedCoordf32, ReverseCoordTranslate}};
use plotters_druid::Plot;

pub struct InteractivePlot<T: Data>(Plot<(T, PlotState)>);

type Cartesian2df32 = Cartesian2d<RangedCoordf32, RangedCoordf32>;

#[derive(Clone)]
pub struct PlotState {
    // Coordinates in data space, not screen space
    cursor_position: Option<(f32, f32)>,
    x_range: RangeIncl<f32>,
    y_range: RangeIncl<f32>,
    coord_spec: Option<Cartesian2df32>,
}

impl Data for PlotState {
    fn same(&self, _other: &Self) -> bool {
        // TODO: False-Positives for inequality are inefficient, but not functionally incorrect

        // self.cursor_position == other.cursor_position
        //     && self.x_range == other.x_range
        //     && self.y_range == other.y_range
        //     && self.coord_spec == other.coord_spec

        false
    }
}

impl PlotState {
    pub fn new() -> Self {
        Self {
            cursor_position: None,
            x_range: RangeIncl::new(0.0, 1.0),
            y_range: RangeIncl::new(0.0, 1.0),
            coord_spec: None,
        }
    }

    pub fn zoom(&mut self, center: (f32, f32), factor: f32) {
        self.x_range.zoom(center.0, factor);
        self.y_range.zoom(center.1, factor);
    }

    pub fn pan(&mut self, delta: (f32, f32)) {
        self.x_range.pan(delta.0);
        self.y_range.pan(delta.1);
    }
}

impl<T: Data> InteractivePlot<T> {
    pub fn new(plot: Plot<(T, PlotState)>) -> Self {
        Self(plot)
    }

    // pub fn new(f: impl Fn((u32, u32), &T, &DrawingArea<PietBackend, Shift>) + 'static) {
    //     Self::new(Plot::new(f))
    // }
}

impl<T: Data> Widget<(T, PlotState)> for InteractivePlot<T> {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut (T, PlotState), env: &druid::Env) {
        let (plot_data, plot_state) = data;

        fn cursor_pos_from_event(state: &mut PlotState, mouse_event: &druid::MouseEvent) -> Option<(f32, f32)> {
            convert_cursor_pos(state, (mouse_event.pos.x as i32, mouse_event.pos.y as i32))
        }

        fn convert_cursor_pos(state: &mut PlotState, cursor_pos_screen: (i32, i32)) -> Option<(f32, f32)> {
            let coord_spec = match state.coord_spec.as_ref() {
                Some(coord_spec) => coord_spec,
                None => return None,
            };
            coord_spec.reverse_translate(cursor_pos_screen)
        }
        // self.0.
        match event {
            druid::Event::MouseMove(mouse_event) => {
                if let Some(cursor_pos_coord) = cursor_pos_from_event(plot_state, mouse_event) {
                    plot_state.cursor_position = Some(cursor_pos_coord);
                    if mouse_event.buttons.contains(druid::MouseButton::Left) {
                        plot_state.pan((cursor_pos_coord.0 - mouse_event.pos.x as f32, cursor_pos_coord.1 - mouse_event.pos.y as f32));
                    }
                    // TODO: Box-Zoom
                }
            },
            druid::Event::Wheel(mouse_event) => {
                if let Some(cursor_pos_coord) = cursor_pos_from_event(plot_state, mouse_event) {
                    let fac = if mouse_event.mods.shift() { 10.0 } else { 30.0 }; // TODO: Make configurable

                    plot_state.zoom(cursor_pos_coord, (mouse_event.wheel_delta.y as f32).exp() * fac);
                }
            },
            _ => {}
        }
        self.0.event(ctx, event, data, env)
    }

    fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, event: &druid::LifeCycle, data: &(T, PlotState), env: &druid::Env) {
        self.0.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &(T, PlotState), data: &(T, PlotState), env: &druid::Env) {
        self.0.update(ctx, old_data, data, env)
    }

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &(T, PlotState), env: &druid::Env) -> druid::Size {
        self.0.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &(T, PlotState), env: &druid::Env) {
        self.0.paint(ctx, data, env)
    }
}