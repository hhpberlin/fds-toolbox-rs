use std::{cell::RefCell, iter::Copied};

use fds_toolbox_core::formats::{
    simulation::SliceSeriesIdx,
    simulations::{SimulationIdx, Simulations},
    slcf::{Slice, SliceFile},
};
use iced::{widget::row, Command, Element};

use crate::{
    plotters::{
        cartesian::{self, cartesian},
        heatmap::Heatmap,
        ids::IdSource,
    },
    tabs::Tab,
};

#[derive(Debug)]
pub struct SliceTab {
    slice: SimulationIdx<SliceSeriesIdx>,
    frame: usize,
    plot_state: RefCell<cartesian::State>,
}

impl IdSource for SliceTab {
    type Id = SimulationIdx<SliceSeriesIdx>;
    type Iter<'a> = Copied<std::slice::Iter<'a, Self::Id>>
    where
        Self: 'a;

    fn iter_ids(&self) -> Self::Iter<'_> {
        [self.slice].iter().copied()
    }
}

pub enum Message {
    Plot(cartesian::Message),
    // AddSlice(SliceFile),
}

impl SliceTab {
    pub fn new(slice: SimulationIdx<SliceSeriesIdx>) -> Self {
        Self {
            slice,
            frame: 0, // TODO
            plot_state: RefCell::new(cartesian::State::default()),
        }
    }
}

impl Tab<Simulations> for SliceTab {
    type Message = Message;

    fn title(&self) -> String {
        "Slice".to_string()
    }

    fn view(&self, model: &Simulations) -> Element<'_, Message> {
        row![
            // Self::view_sidebar(self.series.borrow_mut(), model),
            cartesian(Heatmap::new(model, self.slice), &self.plot_state).map(Message::Plot),
        ]
        .into()
    }

    fn update(
        &mut self,
        _model: &mut Simulations,
        _message: Self::Message,
    ) -> Command<Self::Message> {
        todo!()
    }
}
