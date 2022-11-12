use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator, cell::{RefCell, RefMut}, rc::Rc,
};

use fds_toolbox_core::{
    common::arr_meta::ArrayStats,
    formats::{simulation::TimeSeriesIdx, simulations::GlobalTimeSeriesIdx},
};
use iced::{
    widget::{canvas::Cache, checkbox, row, scrollable, Column},
    Command, Element, Length,
};

use crate::{array_stats_vis::{array_stats_vis}, tabs::Tab, Simulations};

use super::plot::{IdSource, Plot2DState};

#[derive(Debug)]
pub struct PlotTab {
    chart: Plot2DState,
    // selected: HashSet<GlobalTimeSeriesIdx>, // TODO: Should this use HashMap<_, bool> instead>?
    series: RefCell<HashMap<GlobalTimeSeriesIdx, PlotTabSeries>>,
}

#[derive(Debug)]
pub struct PlotTabSeries {
    idx: GlobalTimeSeriesIdx,
    selected: bool,
    array_stats_vis_cache: Cache,
}

impl PlotTabSeries {
    pub fn new(idx: GlobalTimeSeriesIdx) -> Self {
        Self {
            idx,
            selected: false,
            array_stats_vis_cache: Cache::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Plot(super::plot::Message),
    Add(GlobalTimeSeriesIdx),
    Remove(GlobalTimeSeriesIdx),
}

impl PlotTab {
    pub fn new(idx: impl IntoIterator<Item = GlobalTimeSeriesIdx>) -> Self {
        Self {
            chart: Plot2DState::new(),
            // selected: HashSet::from_iter(idx.into_iter()),
            series: RefCell::new(idx
                .into_iter()
                .map(|idx| PlotTabSeries {
                    idx,
                    selected: true,
                    array_stats_vis_cache: Cache::default(),
                })
                .map(|series| (series.idx, series))
                .collect()),
        }
    }

    fn view_sidebar<'a>(
        // set: &'a HashSet<GlobalTimeSeriesIdx>,
        mut series: RefMut<'a, HashMap<GlobalTimeSeriesIdx, PlotTabSeries>>,
        model: &'a Simulations,
    ) -> Element<'a, Message> {
        let mut sidebar = Column::new();

        for (idx, device) in model
            .simulations
            .iter()
            .flat_map(|x| x.devc.enumerate_devices())
        {
            // TODO: This does not work with multiple simulations
            let global_idx = GlobalTimeSeriesIdx(0, TimeSeriesIdx::Device(idx));

            // let info = series.get(&global_idx);
            // let info = info.unwrap_or(default)

            let info = series
                // .get(&global_idx);
                .entry(global_idx)
                .or_insert_with(|| PlotTabSeries::new(global_idx));

            // let info = match info {
            //     Some(info) => info,
            //     None => &PlotTabSeries::new(global_idx)
            // };

            sidebar = sidebar.push(row![
                checkbox(
                    format!("{} ({})", device.name, device.unit),
                    info.selected,
                    move |checked| {
                        if checked {
                            Message::Add(global_idx)
                        } else {
                            Message::Remove(global_idx)
                        }
                    },
                ),
                array_stats_vis(device.values.stats),
                // .push(iced::Text::new(format!("{} ({})", device.name, device.unit)))
            ]);
        }

        scrollable(sidebar).into()
    }
}

impl Tab<Simulations> for PlotTab {
    type Message = Message;

    fn title(&self) -> String {
        // TODO: Give a more descriptive name
        //       Maybe list the names of the selected time series?
        // Sub-TODO: Ellispisize long names? Here or generally?
        "Plot 2D".to_string()
    }

    fn update(
        &mut self,
        _model: &mut Simulations,
        message: Self::Message,
    ) -> Command<Self::Message> {
        match message {
            Message::Plot(msg) => self.chart.update(msg).map(Message::Plot),
            Message::Add(idx) => {
                self.series.borrow_mut().get_mut(&idx).unwrap().selected = true;
                self.chart.invalidate();
                Command::none()
            }
            Message::Remove(idx) => {
                self.series.borrow_mut().get_mut(&idx).unwrap().selected = false;
                self.chart.invalidate();
                Command::none()
            }
        }
    }

    fn view<'a>(&'a self, model: &'a Simulations) -> Element<'a, Self::Message> {
        let ids: Vec<_> = self.series.borrow().iter()
        .filter_map(|(idx, s)| if s.selected { Some(idx) } else { None })
        .copied()
        .collect();

        row![
            Self::view_sidebar(self.series.borrow_mut(), model),
            self.chart.view(model, ids).map(Message::Plot),
        ]
        .into()
    }
}

impl IdSource for HashSet<GlobalTimeSeriesIdx> {
    type Id = GlobalTimeSeriesIdx;

    fn iter_ids(&self) -> Box<dyn Iterator<Item = Self::Id> + '_> {
        Box::new(self.iter().copied())
    }
}

impl IdSource for Vec<GlobalTimeSeriesIdx> {
    type Id = GlobalTimeSeriesIdx;

    fn iter_ids(&self) -> Box<dyn Iterator<Item = Self::Id> + '_> {
        Box::new(self.iter().copied())
    }
}

impl IdSource for HashMap<GlobalTimeSeriesIdx, PlotTabSeries> {
    type Id = GlobalTimeSeriesIdx;

    fn iter_ids(&self) -> Box<dyn Iterator<Item = Self::Id> + '_> {
        Box::new(
            self.iter()
                .filter_map(|(idx, s)| if s.selected { Some(idx) } else { None })
                .copied(),
        )
    }
}
