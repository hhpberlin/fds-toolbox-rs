use iced::{Command, Element};

use crate::FdsToolboxData;

use self::plot_tab::PlotTab;

mod plot;
pub mod plot_tab;

pub trait Tab<Model> {
    type Message;

    fn title(&self) -> String;

    fn update(&mut self, model: &mut Model, message: Self::Message) -> Command<Self::Message>;
    fn view(&mut self, model: &Model) -> Element<'_, Self::Message>;
}

// TODO: This is very boilerplate-y

#[derive(Debug)]
pub enum FdsToolboxTab {
    Overview(PlotTab),
}

#[derive(Debug, Clone, Copy)]
pub enum FdsToolboxTabMessage {
    Overview(<PlotTab as Tab<FdsToolboxData>>::Message),
}

impl Tab<FdsToolboxData> for FdsToolboxTab {
    type Message = FdsToolboxTabMessage;

    fn title(&self) -> String {
        match self {
            FdsToolboxTab::Overview(tab) => tab.title(),
        }
    }

    fn update(
        &mut self,
        model: &mut FdsToolboxData,
        message: Self::Message,
    ) -> Command<Self::Message> {
        match (self, message) {
            (FdsToolboxTab::Overview(tab), FdsToolboxTabMessage::Overview(msg)) => {
                tab.update(model, msg).map(FdsToolboxTabMessage::Overview)
            }
            _ => {
                // TODO: Log error
                Command::none()
            }
        }
    }

    fn view(&mut self, model: &FdsToolboxData) -> Element<'_, Self::Message> {
        match self {
            FdsToolboxTab::Overview(tab) => tab.view(model).map(FdsToolboxTabMessage::Overview),
        }
    }
}
