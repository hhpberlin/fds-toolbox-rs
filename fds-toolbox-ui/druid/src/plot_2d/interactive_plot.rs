use std::rc::Rc;

use druid::{Data, Widget};
use fds_toolbox_core::common::{range::RangeIncl, series::TimeSeriesView};
use plotters::{
    coord::{types::RangedCoordf32, ReverseCoordTranslate},
    prelude::{Cartesian2d, ChartBuilder},
    series::LineSeries,
    style::RED,
};
use plotters_druid::Plot;

pub struct InteractivePlot<T: Data> {
    plot: Plot<(T, PlotState, DataSource<T>)>,
    state: PlotState,
    data_source: DataSource<T>,
}

// pub struct InteractivePlotData<T: Data> {
//     pub data: T,
//     pub state: PlotState,
//     pub data_source: DataSource<T>,
// }

type Cartesian2df32 = Cartesian2d<RangedCoordf32, RangedCoordf32>;

#[derive(Clone, Data)]
pub struct DataSource<T>(pub Rc<dyn for<'a> Fn(&'a T) -> MultiSeriesView<'a> + 'static>);

// #[derive(Clone)]
pub struct MultiSeriesView<'a>(pub Box<dyn Iterator<Item = TimeSeriesView<'a>> + 'a>);

#[derive(Clone)]
pub struct PlotState {
    // Coordinates in data space, not screen space
    pub cursor_position: Option<(f32, f32)>,
    pub x_range: RangeIncl<f32>,
    pub y_range: RangeIncl<f32>,
    pub coord_spec: Option<Cartesian2df32>,
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
    pub fn new(data_source: DataSource<T>) -> Self {
        Self {
            state: PlotState::new(),
            data_source,
            plot: Plot::<(T, PlotState, DataSource<T>)>::new(|(width, height), (data, plot_state, data_source), root| {
                let data = data_source.0(data);

                let mut chart = ChartBuilder::on(&root)
                    .margin(5)
                    .x_label_area_size(30)
                    .y_label_area_size(30)
                    .build_cartesian_2d(
                        plot_state.x_range.into_range(),
                        plot_state.y_range.into_range(),
                    )
                    .unwrap();

                chart.configure_mesh().draw().unwrap();

                for series in data.0 {
                    chart
                        .draw_series(LineSeries::new(series.iter(), RED))
                        .unwrap();
                }

                // self.plot_state.coord_spec = Some(*chart.as_coord_spec());
            }),
        }
    }

    // pub fn new(plot: Plot<(T, PlotState)>) -> Self {
    //     Self { plot, state: PlotState::new(), data_source: Box::new(|_| MultiSeriesView(Box::new(std::iter::empty()))) }
    // }

    // pub fn new(f: impl Fn((u32, u32), &T, &DrawingArea<PietBackend, Shift>) + 'static) {
    //     Self::new(Plot::new(f))
    // }
}

impl<T: Data> Widget<T> for InteractivePlot<T> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut T,
        env: &druid::Env,
    ) {
        fn cursor_pos_from_event(
            state: &mut PlotState,
            mouse_event: &druid::MouseEvent,
        ) -> Option<(f32, f32)> {
            convert_cursor_pos(state, (mouse_event.pos.x as i32, mouse_event.pos.y as i32))
        }

        fn convert_cursor_pos(
            state: &mut PlotState,
            cursor_pos_screen: (i32, i32),
        ) -> Option<(f32, f32)> {
            let coord_spec = match state.coord_spec.as_ref() {
                Some(coord_spec) => coord_spec,
                None => return None,
            };
            coord_spec.reverse_translate(cursor_pos_screen)
        }
        // self.0.
        match event {
            druid::Event::MouseMove(mouse_event) => {
                if let Some(cursor_pos_coord) = cursor_pos_from_event(&mut self.state, mouse_event)
                {
                    self.state.cursor_position = Some(cursor_pos_coord);
                    if mouse_event.buttons.contains(druid::MouseButton::Left) {
                        self.state.pan((
                            cursor_pos_coord.0 - mouse_event.pos.x as f32,
                            cursor_pos_coord.1 - mouse_event.pos.y as f32,
                        ));
                    }
                    // TODO: Box-Zoom
                }
            }
            druid::Event::Wheel(mouse_event) => {
                if let Some(cursor_pos_coord) = cursor_pos_from_event(&mut self.state, mouse_event)
                {
                    let fac = if mouse_event.mods.shift() { 10.0 } else { 30.0 }; // TODO: Make configurable

                    self.state.zoom(
                        cursor_pos_coord,
                        (mouse_event.wheel_delta.y as f32).exp() * fac,
                    );
                }
            }
            _ => {}
        }
        self.plot.event(ctx, event, &mut (data.clone(), self.state.clone(), self.data_source.clone()), env)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &T,
        env: &druid::Env,
    ) {
        self.plot.lifecycle(ctx, event, &(data.clone(), self.state.clone(), self.data_source.clone()), env)
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &T, data: &T, env: &druid::Env) {
        if data.same(old_data) { return; }

        self.plot
            .update(ctx, &(old_data.clone(), self.state.clone(), self.data_source.clone()), &(data.clone(), self.state.clone(), self.data_source.clone()), env)
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &druid::Env,
    ) -> druid::Size {
        self.plot.layout(ctx, bc, &(data.clone(), self.state.clone(), self.data_source.clone()), env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &T, env: &druid::Env) {
        self.plot.paint(ctx, &(data.clone(), self.state.clone(), self.data_source.clone()), env)
    }
}
