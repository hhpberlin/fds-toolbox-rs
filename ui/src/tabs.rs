use iced::{Command, Element};

use crate::{plot_2d::plot_tab::PlotTab, slice::slice_tab::SliceTab, Model};

pub trait Tab {
    type Message;

    fn title(&self) -> String;

    fn update(&mut self, model: &mut Model, message: Self::Message) -> Command<Self::Message>;
    fn view<'a>(&'a self, model: &'a Model) -> Element<'a, Self::Message>;
    // fn invalidate(&mut self);
}

// TODO: This is very boilerplate-y
//       Use enum_dispatch?
//       Just Box<dyn Tab<Simulations>> kind of stuff? How would message types work seeing as they're associated types?

#[derive(Debug)]
pub enum FdsToolboxTab {
    Plot(PlotTab),
    Slice(SliceTab),
}

#[derive(Debug, Clone)]
pub enum FdsToolboxTabMessage {
    Plot(<PlotTab as Tab>::Message),
    Slice(<SliceTab as Tab>::Message),
}

impl Tab for FdsToolboxTab {
    type Message = FdsToolboxTabMessage;

    fn title(&self) -> String {
        match self {
            FdsToolboxTab::Plot(tab) => Tab::title(tab),
            FdsToolboxTab::Slice(tab) => Tab::title(tab),
        }
    }

    fn update(&mut self, model: &mut Model, message: Self::Message) -> Command<Self::Message> {
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

    fn view<'a>(&'a self, model: &'a Model) -> Element<'a, Self::Message> {
        match self {
            FdsToolboxTab::Plot(tab) => tab.view(model).map(FdsToolboxTabMessage::Plot),
            FdsToolboxTab::Slice(tab) => tab.view(model).map(FdsToolboxTabMessage::Slice),
        }
    }

    // fn invalidate(&mut self) {
    //     match self {
    //         FdsToolboxTab::Plot(tab) => tab.invalidate(),
    //         FdsToolboxTab::Slice(tab) => tab.invalidate(),
    //     }
    // }
}
