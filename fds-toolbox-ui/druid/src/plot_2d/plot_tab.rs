use std::{collections::HashSet, rc::Rc};

use druid::{Data, Lens, Widget};
use fds_toolbox_core::{
    common::series::TimeSeriesViewSource, formats::simulations::GlobalTimeSeriesIdx,
};

use crate::{
    state::{FdsToolboxData},
    tab::Tab,
};

use super::interactive_plot::{DataSource, InteractivePlot, MultiSeriesView};

pub struct Plot2DTab {
    // selected: HashSet<GlobalTimeSeriesIdx>,
}

impl Plot2DTab {
    pub fn new() -> Self {
        Self {
            // selected: HashSet::new(),
            // plot_state: PlotState::new(),
        }
    }
}

#[derive(Clone, Lens)]
pub struct Plot2DTabData {
    pub selected: HashSet<GlobalTimeSeriesIdx>,
}

impl Plot2DTabData {
    pub fn new(selected: HashSet<GlobalTimeSeriesIdx>) -> Self {
        Self { selected }
    }
}

impl Data for Plot2DTabData {
    fn same(&self, other: &Self) -> bool {
        self.selected.eq(&other.selected)
    }
}

impl Tab<FdsToolboxData> for Plot2DTab {
    type Data = Plot2DTabData;

    fn title(&self) -> String {
        "Plot 2D".to_string()
    }

    fn build_widget(&mut self) -> Box<dyn Widget<(Plot2DTabData, FdsToolboxData)>> {
        Box::new(InteractivePlot::new(DataSource(Rc::new(
            |(tab, data): &(Self::Data, FdsToolboxData)| {
                let iter = tab
                    .selected
                    .iter()
                    .filter_map(|idx| data.simulations.get_time_series(*idx));

                MultiSeriesView(Box::new(iter))
            },
        )) as _))
    }
}
