use std::collections::HashSet;

use druid::{Widget, WidgetExt, Lens};
use fds_toolbox_core::{formats::simulations::GlobalTimeSeriesIdx, common::series::{TimeSeriesView, TimeSeriesViewSource}};
use plotters::{style::RED, prelude::ChartBuilder, series::LineSeries};
use plotters_druid::Plot;

use crate::{tab::Tab, state::FdsToolboxApp};

use super::interactive_plot::{PlotState, InteractivePlot};

struct Plot2DTab {
    selected: HashSet<GlobalTimeSeriesIdx>,
    plot_state: PlotState,
}

impl Plot2DTab {
    fn new() -> Self {
        Self {
            selected: HashSet::new(),
            plot_state: PlotState::new(),
        }
    }
}

type MultiSeriesView<'a> = Box<dyn Iterator<Item = TimeSeriesView<'a>> + 'a>;

impl<'a> Lens<FdsToolboxApp, MultiSeriesView<'a>> for HashSet<GlobalTimeSeriesIdx> {
    fn with<V, F: FnOnce(&MultiSeriesView<'a>) -> V>(&self, data: &FdsToolboxApp, f: F) -> V {
        f(&Box::new(self.iter()
            .filter_map(|idx| data.simulations.get_time_series(*idx))) as _)
    }

    fn with_mut<V, F: FnOnce(&mut MultiSeriesView<'a>) -> V>(&self, data: &mut FdsToolboxApp, f: F) -> V {
        todo!()
    }
}

impl Tab<FdsToolboxApp> for Plot2DTab {
    fn title(&self) -> String {
        "Plot 2D".to_string()
    }

    fn build_widget(&mut self) -> Box<dyn Widget<(Self, FdsToolboxApp)>> {
        Box::new(InteractivePlot::new(
            Plot::new(|(data, plot_state), _env| {
                let (width, height) = data.size();
                let (x_range, y_range) = plot_state.get_ranges();
                let coord_spec = plot_state.get_coord_spec(width, height);

                let mut chart = ChartBuilder::on(&data)
                    .margin(5)
                    .x_label_area_size(30)
                    .y_label_area_size(30)
                    .build_cartesian_2d(x_range, y_range)
                    .unwrap();

                chart
                    .configure_mesh()
                    // .disable_x_mesh()
                    // .disable_y_mesh()
                    // .x_desc("X")
                    // .y_desc("Y")
                    .draw()
                    .unwrap();

                chart
                    .draw_series(LineSeries::new(
                        data.iter().map(|(x, y)| (*x, *y)),
                        &RED,
                    ))
                    .unwrap();
            }),
        ).lens(self.selected) as _)
    }
}