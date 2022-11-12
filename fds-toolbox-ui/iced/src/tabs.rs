use iced::{Command, Element};

use crate::{plot_2d::plot_tab::PlotTab, Simulations};

pub trait Tab<Model> {
    type Message;

    fn title(&self) -> String;

    fn update(&mut self, model: &mut Model, message: Self::Message) -> Command<Self::Message>;
    fn view<'a>(&'a self, model: &'a Model) -> Element<'a, Self::Message>;
}

// TODO: This is very boilerplate-y

#[derive(Debug)]
pub enum FdsToolboxTab {
    Overview(PlotTab),
}

#[derive(Debug, Clone, Copy)]
pub enum FdsToolboxTabMessage {
    Overview(<PlotTab as Tab<Simulations>>::Message),
}

impl Tab<Simulations> for FdsToolboxTab {
    type Message = FdsToolboxTabMessage;

    fn title(&self) -> String {
        match self {
            FdsToolboxTab::Overview(tab) => tab.title(),
        }
    }

    fn update(
        &mut self,
        model: &mut Simulations,
        message: Self::Message,
    ) -> Command<Self::Message> {
        match (self, message) {
            (FdsToolboxTab::Overview(tab), FdsToolboxTabMessage::Overview(msg)) => {
                tab.update(model, msg).map(FdsToolboxTabMessage::Overview)
            }
        }
    }

    fn view<'a>(&'a self, model: &'a Simulations) -> Element<'a, Self::Message> {
        match self {
            FdsToolboxTab::Overview(tab) => tab.view(model).map(FdsToolboxTabMessage::Overview),
        }
    }
}
