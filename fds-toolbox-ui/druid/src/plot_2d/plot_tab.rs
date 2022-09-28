use std::{collections::HashSet, rc::Rc};

use druid::{Widget, WidgetExt, Lens, Data};
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

#[derive(Clone)]
struct MultiSeriesView<'a>(Rc<dyn Iterator<Item = TimeSeriesView<'a>> + 'a>);

impl<'a> Lens<FdsToolboxApp, (MultiSeriesView<'a>, PlotState)> for &mut Plot2DTab {
    fn with<V, F: FnOnce(&(MultiSeriesView<'a>, PlotState)) -> V>(&self, data: &FdsToolboxApp, f: F) -> V {
        let iter = self.selected.iter()
            .filter_map(|idx| data.simulations.get_time_series(*idx));

        f(&(MultiSeriesView(Box::new(iter) as _), self.plot_state))
    }

    fn with_mut<V, F: FnOnce(&mut (MultiSeriesView<'a>, PlotState)) -> V>(&self, data: &mut FdsToolboxApp, f: F) -> V {
        let iter = self.selected.iter()
            .filter_map(|idx| data.simulations.get_time_series(*idx));

        f(&mut (MultiSeriesView(Box::new(iter) as _), self.plot_state))
    }
}

impl Data for MultiSeriesView<'_> {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}
// impl<T, U, F: Fn(&T) -> &U, FMut: FnMut(&mut T) -> &mut U> Lens<T,


impl Tab<FdsToolboxApp> for Plot2DTab {
    fn title(&self) -> String {
        "Plot 2D".to_string()
    }

    fn build_widget(&mut self) -> Box<dyn Widget<(Self, FdsToolboxApp)>> {
        Box::new(InteractivePlot::new(
            Plot::new(|(width, height), (data, plot_state): (MultiSeriesView<'_>, PlotState), root| {
                // let coord_spec = plot_state.coord_spec;

                let mut chart = ChartBuilder::on(&root)
                    .margin(5)
                    .x_label_area_size(30)
                    .y_label_area_size(30)
                    .build_cartesian_2d(plot_state.x_range.into_range(), plot_state.y_range.into_range())
                    .unwrap();

                    chart
                    .configure_mesh()
                    // .disable_x_mesh()
                    // .disable_y_mesh()
                    // .x_desc("X")
                    // .y_desc("Y")
                    .draw()
                    .unwrap();
                    
                    for series in data {
                        chart.draw_series(LineSeries::new(series.iter(), RED)).unwrap();
                    }

                self.plot_state.coord_spec = Some(*chart.as_coord_spec());
            }),
        ).lens(self) as _)
    }
}