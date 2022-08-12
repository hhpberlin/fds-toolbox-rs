use enum_dispatch::enum_dispatch;
use iced::{Command, Element};

use crate::FdsToolbox;

pub struct OverviewTab;

impl Tab<FdsToolbox> for OverviewTab {
    type Message = ();

    fn title(&self) -> String {
        todo!()
    }

    fn update(&mut self, model: &mut FdsToolbox, message: Self::Message) -> Command<Self::Message> {
        todo!()
    }

    fn view(&mut self, model: &mut FdsToolbox) -> Element<'_, Self::Message> {
        todo!()
    }
}

pub trait Tab<Model> {
    type Message;

    fn title(&self) -> String;

    fn update(&mut self, model: &mut Model, message: Self::Message) -> Command<Self::Message>;
    fn view(&mut self, model: &mut Model) -> Element<'_, Self::Message>;
}

pub enum FdsToolboxTab {
    Overview(OverviewTab),
}

type FdsToolboxTab = Box<dyn Tab<FdsToolbox, Message = ()>>;

impl Tab<FdsToolbox> for FdsToolboxTab {
    type Message = FdsToolboxMessage;

    fn title(&self) -> String {
        match self {
            FdsToolboxTab::Overview(tab) => tab.title(),
        }
    }

    fn update(&mut self, model: &mut FdsToolbox, message: Self::Message) -> Command<Self::Message> {
        match self {
            FdsToolboxTab::Overview(tab) => FdsToolboxMessage::Overview(tab.update(model, message)),
        }
    }

    fn view(&mut self, model: &mut FdsToolbox) -> Element<'_, Self::Message> {
        match self {
            FdsToolboxTab::Overview(tab) => tab.view(model),
        }
    }
}