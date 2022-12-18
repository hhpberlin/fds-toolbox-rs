use std::{
    cell::RefCell,
    iter::{once, Once},
};

use fds_toolbox_core::formats::{
    simulation::SliceSeriesIdx,
    simulations::{SimulationIdx, Simulations},
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
    type Iter<'a> = Once<Self::Id>
    where
        Self: 'a;

    fn iter_ids(&self) -> Self::Iter<'_> {
        once(self.slice)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Plot(cartesian::Message),
    // AddSlice(SliceFile),
}

impl SliceTab {
    pub fn new(slice: SimulationIdx<SliceSeriesIdx>) -> Self {
        Self {
            slice,
            frame: 500, // TODO
            plot_state: RefCell::new(cartesian::State::new(
                (0.0..=10.0).into(),
                (0.0..=10.0).into(),
            )),
        }
    }
}

impl Tab<Simulations> for SliceTab {
    type Message = Message;

    fn title(&self) -> String {
        "Slice Plot".to_string()
    }

    fn view<'a>(&'a self, model: &'a Simulations) -> Element<'a, Message> {
        row![
            // Self::view_sidebar(self.series.borrow_mut(), model),
            cartesian(Heatmap::new(model, self, self.frame), &self.plot_state).map(Message::Plot),
        ]
        .into()
    }

    fn update(
        &mut self,
        _model: &mut Simulations,
        _message: Self::Message,
    ) -> Command<Self::Message> {
        match _message {
            Message::Plot(_) => {
                // self.plot_state.borrow_mut().update(msg);
                Command::none()
            }
        }
    }
}
