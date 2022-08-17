use iced::{Command, Element, Text};

use crate::FdsToolboxData;

use super::Tab;

#[derive(Debug)]
pub struct OverviewTab {
}

impl OverviewTab {
    pub fn new() -> Self {
        Self {

        }
    }
}

impl Tab<FdsToolboxData> for OverviewTab {
    type Message = ();

    fn title(&self) -> String {
        "Overview".to_string()
    }

    fn update(
        &mut self,
        _model: &mut FdsToolboxData,
        _message: Self::Message,
    ) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self, model: &FdsToolboxData) -> Element<'_, Self::Message> {
        let devc = &model.simulations[0].devc;
        Text::new("Overview").size(20).into()
    }
}
