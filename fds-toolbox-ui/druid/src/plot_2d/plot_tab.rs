use std::{collections::HashSet, rc::Rc};

use druid::{widget::Scroll, Data, Lens, Widget, WidgetExt};
use fds_toolbox_core::{
    common::series::{TimeSeriesView, TimeSeriesViewSource},
    formats::simulations::GlobalTimeSeriesIdx,
};
use plotters::{prelude::ChartBuilder, series::LineSeries, style::RED};
use plotters_druid::Plot;

use crate::{state::FdsToolboxApp, tab::Tab};

use super::interactive_plot::{InteractivePlot, MultiSeriesView, PlotState, DataSource};

#[derive(Clone, Lens)]
struct Plot2DTab {
    selected: HashSet<GlobalTimeSeriesIdx>,
}

impl Data for Plot2DTab {
    fn same(&self, other: &Self) -> bool {
        self.selected.eq(&other.selected)
    }
}

impl Plot2DTab {
    fn new() -> Self {
        Self {
            selected: HashSet::new(),
            // plot_state: PlotState::new(),
        }
    }
}

impl Tab<FdsToolboxApp> for Plot2DTab {
    type Data = Plot2DTab;

    fn title(&self) -> String {
        "Plot 2D".to_string()
    }

    fn build_widget(&mut self) -> Box<dyn Widget<(Self, FdsToolboxApp)>> {
        Box::new(InteractivePlot::new(DataSource(Rc::new(|(tab, data): &(Self, FdsToolboxApp)| {
            let iter = tab
                .selected
                .iter()
                .filter_map(|idx| data.simulations.get_time_series(*idx));

            MultiSeriesView(Box::new(iter))
        })) as _))
    }
}
