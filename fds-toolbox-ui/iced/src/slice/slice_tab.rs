use fds_toolbox_core::formats::{
    simulation::SliceIdx,
    simulations::{SimulationIdx, Simulations},
    slcf::{Slice, SliceFile},
};
use iced::{Command, Element};

use crate::tabs::Tab;

#[derive(Debug)]
pub struct SliceTab {
    slice: SimulationIdx<SliceIdx>,
}

pub enum Message {
    // AddSlice(SliceFile),
}

impl SliceTab {
    pub fn new(slice: Slice) -> Self {
        Self {
            slice: Slice::new(slice),
        }
    }
}

impl Tab<Simulations> for SliceTab {
    type Message = Message;

    fn title(&self) -> String {
        todo!()
    }

    fn view(&self, _model: &Simulations) -> Element<'_, Message> {
        todo!()
    }

    fn update(
        &mut self,
        _model: &mut Simulations,
        _message: Self::Message,
    ) -> Command<Self::Message> {
        todo!()
    }
}
