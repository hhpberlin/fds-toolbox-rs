use std::{collections::HashSet, iter::FromIterator};

use fds_toolbox_core::formats::{simulation::TimeSeriesIdx, simulations::GlobalTimeSeriesIdx};
use iced::{scrollable, Column, Command, Element, Row, Scrollable};

use crate::{tabs::Tab, Simulations};

use super::plot::{IdSource, Plot2DState};

#[derive(Debug)]
pub struct PlotTab {
    chart: Plot2DState,
    scrollable: scrollable::State,
    selected: HashSet<GlobalTimeSeriesIdx>, // TODO: Should this use HashMap<_, bool> instead>?
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
            scrollable: scrollable::State::new(),
            selected: HashSet::from_iter(idx.into_iter()),
        }
    }

    fn view_sidebar<'a>(
        set: &'a HashSet<GlobalTimeSeriesIdx>,
        scroll: &'a mut scrollable::State,
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

            sidebar = sidebar.push(
                Row::new().push(iced::Checkbox::new(
                    set.contains(&global_idx),
                    format!("{} ({})", device.name, device.unit),
                    move |checked| {
                        if checked {
                            Message::Add(global_idx)
                        } else {
                            Message::Remove(global_idx)
                        }
                    },
                )), // .push(iced::Text::new(format!("{} ({})", device.name, device.unit)))
            );
        }

        Scrollable::new(scroll).push(sidebar).into()
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
                self.selected.insert(idx);
                self.chart.invalidate();
                Command::none()
            }
            Message::Remove(idx) => {
                self.selected.remove(&idx);
                self.chart.invalidate();
                Command::none()
            }
        }
    }

    fn view<'a>(&'a mut self, model: &'a Simulations) -> Element<'a, Self::Message> {
        Row::new()
            .push(Self::view_sidebar(
                &self.selected,
                &mut self.scrollable,
                model,
            ))
            .push(self.chart.view(model, &self.selected).map(Message::Plot))
            .into()
    }
}

impl IdSource for HashSet<GlobalTimeSeriesIdx> {
    type Id = GlobalTimeSeriesIdx;

    fn iter_ids(&self) -> Box<dyn Iterator<Item = Self::Id> + '_> {
        Box::new(self.iter().copied())
    }
}
