use std::{
    cell::{RefCell, RefMut},
    collections::{HashMap, HashSet},
    iter::Copied,
};

use fds_toolbox_core::formats::{simulation::TimeSeriesIdx, simulations::SimulationIdx};
use iced::{
    widget::{canvas::Cache, checkbox, container, horizontal_space, row, scrollable, Column},
    Command, Element, Length,
};

use crate::{
    array_stats_vis::array_stats_vis,
    plotters::{
        cartesian::{self, cartesian},
        ids::IdSource,
        lines::LinePlot,
    },
    tabs::Tab,
    Simulations,
};

#[derive(Debug)]
pub struct PlotTab {
    // chart: CartesianPlot<LinePlot<GlobalTimeSeriesIdx, Simulations, HashMap<GlobalTimeSeriesIdx, PlotTabSeries>>>,
    // selected: HashSet<GlobalTimeSeriesIdx>, // TODO: Should this use HashMap<_, bool> instead>?
    series: RefCell<HashMap<SimulationIdx<TimeSeriesIdx>, PlotTabSeries>>,
    plot_state: RefCell<cartesian::State>,
}

#[derive(Debug)]
pub struct PlotTabSeries {
    idx: SimulationIdx<TimeSeriesIdx>,
    selected: bool,
    array_stats_vis_cache: Cache,
}

impl PlotTabSeries {
    pub fn new(idx: SimulationIdx<TimeSeriesIdx>) -> Self {
        Self {
            idx,
            selected: false,
            array_stats_vis_cache: Cache::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Plot(cartesian::Message),
    Add(SimulationIdx<TimeSeriesIdx>),
    Remove(SimulationIdx<TimeSeriesIdx>),
}

impl PlotTab {
    pub fn new(idx: impl IntoIterator<Item = SimulationIdx<TimeSeriesIdx>>) -> Self {
        Self {
            // chart: CartesianPlot::new(LinePlot::new()),
            series: RefCell::new(
                idx.into_iter()
                    .map(|idx| PlotTabSeries {
                        idx,
                        selected: true,
                        array_stats_vis_cache: Cache::default(),
                    })
                    .map(|series| (series.idx, series))
                    .collect(),
            ),
            plot_state: RefCell::new(cartesian::State::new(
                (0.0..=100.0).into(),
                (0.0..=100.0).into(),
            )),
        }
    }

    fn view_sidebar<'a>(
        mut series: RefMut<'a, HashMap<SimulationIdx<TimeSeriesIdx>, PlotTabSeries>>,
        model: &'a Simulations,
    ) -> Element<'a, Message> {
        let mut sidebar = Column::new();

        for (idx, device) in model
            .simulations
            .iter()
            .flat_map(|x| x.devc.enumerate_devices())
        {
            // TODO: This does not work with multiple simulations
            let global_idx = SimulationIdx(0, TimeSeriesIdx::Device(idx));

            let info = series
                .entry(global_idx)
                .or_insert_with(|| PlotTabSeries::new(global_idx));

            sidebar = sidebar
                .push(row![
                    container(checkbox(
                        format!("{} ({})", device.name, device.unit),
                        info.selected,
                        move |checked| {
                            if checked {
                                Message::Add(global_idx)
                            } else {
                                Message::Remove(global_idx)
                            }
                        },
                    ))
                    .width(Length::Shrink),
                    horizontal_space(Length::Fill),
                    container(array_stats_vis(device.values.stats))
                        .width(Length::Fixed(100.))
                        .height(Length::Fixed(20.)),
                ])
                .max_width(400);
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
        "Line Plot".to_string()
    }

    fn update(
        &mut self,
        _model: &mut Simulations,
        message: Self::Message,
    ) -> Command<Self::Message> {
        // self.chart.invalidate();
        match message {
            Message::Plot(_) => Command::none(), //self.chart.update(msg).map(Message::Plot),
            Message::Add(idx) => {
                self.series.borrow_mut().get_mut(&idx).unwrap().selected = true;
                Command::none()
            }
            Message::Remove(idx) => {
                self.series.borrow_mut().get_mut(&idx).unwrap().selected = false;
                Command::none()
            }
        }
    }

    fn view<'a>(&'a self, model: &'a Simulations) -> Element<'a, Self::Message> {
        let ids: Vec<_> = self.series.borrow().iter_ids().collect();

        row![
            Self::view_sidebar(self.series.borrow_mut(), model),
            cartesian(LinePlot::new(model, ids), &self.plot_state).map(Message::Plot),
        ]
        .into()
    }
}

impl IdSource for HashSet<SimulationIdx<TimeSeriesIdx>> {
    type Id = SimulationIdx<TimeSeriesIdx>;
    // The things I do to avoid a single alloc lmao
    type Iter<'a> = Copied<std::collections::hash_set::Iter<'a, Self::Id>>;

    fn iter_ids(&self) -> Self::Iter<'_> {
        self.iter().copied()
    }
}

impl IdSource for Vec<SimulationIdx<TimeSeriesIdx>> {
    type Id = SimulationIdx<TimeSeriesIdx>;
    type Iter<'a> = Copied<std::slice::Iter<'a, Self::Id>>;

    fn iter_ids(&self) -> Self::Iter<'_> {
        self.iter().copied()
    }
}

impl IdSource for HashMap<SimulationIdx<TimeSeriesIdx>, PlotTabSeries> {
    type Id = SimulationIdx<TimeSeriesIdx>;
    // TODO: Find a way to avoid this alloc
    //       Currently here because filter_map's iterator cannot be named
    type Iter<'a> = Box<dyn Iterator<Item = Self::Id> + 'a>;

    fn iter_ids(&self) -> Self::Iter<'_> {
        Box::new(
            self.iter()
                .filter_map(|(idx, s)| if s.selected { Some(idx) } else { None })
                .copied(),
        )
    }
}
