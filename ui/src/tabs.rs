use fds_toolbox_lazy_data::fs::AnyFs;
use iced::{Command, Element};

use crate::{plot_2d::plot_tab::PlotTab, slice::slice_tab::SliceTab, Simulations};

pub trait Tab<Model> {
    type Message;

    fn title(&self) -> String;

    fn update(&mut self, model: &mut Model, message: Self::Message) -> Command<Self::Message>;
    fn view<'a>(&'a self, model: &'a Model) -> Element<'a, Self::Message>;
}

// TODO: This is very boilerplate-y
//       Use enum_dispatch?
//       Just Box<dyn Tab<Simulations>> kind of stuff? How would message types work seeing as they're associated types?

#[derive(Debug)]
pub enum FdsToolboxTab {
    Plot(PlotTab),
    Slice(SliceTab),
}

#[derive(Debug, Clone, Copy)]
pub enum FdsToolboxTabMessage {
    Plot(<PlotTab as Tab<Simulations<AnyFs>>>::Message),
    Slice(<SliceTab as Tab<Simulations<AnyFs>>>::Message),
}

impl Tab<Simulations<AnyFs>> for FdsToolboxTab {
    type Message = FdsToolboxTabMessage;

    fn title(&self) -> String {
        match self {
            FdsToolboxTab::Plot(tab) => Tab::title(tab),
            FdsToolboxTab::Slice(tab) => Tab::title(tab),
        }
    }

    fn update(
        &mut self,
        model: &mut Simulations<AnyFs>,
        message: Self::Message,
    ) -> Command<Self::Message> {
        match (self, message) {
            (FdsToolboxTab::Plot(tab), FdsToolboxTabMessage::Plot(msg)) => {
                tab.update(model, msg).map(FdsToolboxTabMessage::Plot)
            }
            (FdsToolboxTab::Slice(tab), FdsToolboxTabMessage::Slice(msg)) => {
                tab.update(model, msg).map(FdsToolboxTabMessage::Slice)
            }
            (_tab, _msg) => {
                // TODO: Actually do stuff here

                // panic!("Unhandled message: {:?} for tab: {:?}", msg, tab);

                // log::warn!("Unhandled message: {:?} for tab: {:?}", msg, tab);
                Command::none()
            }
        }
    }

    fn view<'a>(&'a self, model: &'a Simulations<AnyFs>) -> Element<'a, Self::Message> {
        match self {
            FdsToolboxTab::Plot(tab) => tab.view(model).map(FdsToolboxTabMessage::Plot),
            FdsToolboxTab::Slice(tab) => tab.view(model).map(FdsToolboxTabMessage::Slice),
        }
    }
}
