use fds_toolbox_core::formats::{simulations::Simulations, slcf::SliceFile};
use iced::{Command, Element};

use crate::tabs::Tab;

#[derive(Debug)]
pub struct SliceTab {
    // slice: Slice,
}

pub enum Message {
    // AddSlice(SliceFile),
}

impl SliceTab {
    pub fn new(slice: SliceFile) -> Self {
        Self {
            // slice: Slice::new(slice),
        }
    }
}

impl Tab<Simulations> for SliceTab {
    type Message = Message;

    fn title(&self) -> String {
        todo!()
    }

    fn view(&self, model: &Simulations) -> Element<'_, Message> {
        todo!()
    }

    fn update(
        &mut self,
        model: &mut Simulations,
        message: Self::Message,
    ) -> Command<Self::Message> {
        todo!()
    }
}
