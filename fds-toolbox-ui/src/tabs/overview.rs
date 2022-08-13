use iced::{Command, Element, Text};

use crate::{FdsToolbox, FdsToolboxData};

use super::Tab;

#[derive(Debug)]
pub struct OverviewTab;

impl Tab<FdsToolboxData> for OverviewTab {
    type Message = ();

    fn title(&self) -> String {
        "Overview".to_string()
    }

    fn update(
        &mut self,
        model: &mut FdsToolboxData,
        message: Self::Message,
    ) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self, model: &mut FdsToolboxData) -> Element<'_, Self::Message> {
        Text::new("Overview").size(20).into()
    }
}
