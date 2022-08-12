use std::any::Any;

use iced::{Application, Settings, executor, Command, Element, Alignment, Length};
use iced::widget::{Column, Text};
use iced_aw::{TabBar, TabLabel};
use tab::{Tab, FdsToolboxTab};

mod panes;
mod tab;

pub fn main() -> iced::Result {
    FdsToolbox::run(Settings::default())
}

struct FdsToolbox {
    active_tab: Option<usize>,
    tabs: Vec<FdsToolboxTab>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    TabSelected(usize),
    TabClosed(usize),
}

impl FdsToolbox {
    // fn get(&self, index: usize) -> Option<&dyn Tab<FdsToolbox, Message = Box<dyn Any>>> {
    //     if index == 0 {
    //         Some(&self.overview_tab)
    //     } else {
    //         self.tabs.get(index - 1)
    //     }
    // }
}

impl Application for FdsToolbox {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            FdsToolbox {
                active_tab: None,
                tabs: Vec::new(),
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        "FDS Toolbox".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        // Column::new()
        //     .push(Text::new("FDS Toolbox").size(50))
        //     .padding(20)
        //     .align_items(Alignment::Center)
        //     .into()

        let tab_bar = self.active_tab.map(|tab| (
            self.tabs
                .iter()
                .fold(
                    TabBar::new(tab, Message::TabSelected),
                    |tab_bar, tab| {
                        let tab_label = <FdsToolboxTab as Tab<FdsToolbox>>::title(tab);
                        tab_bar.push(TabLabel::Text(tab_label))
                    },
                )
                .on_close(Message::TabClosed)
                .tab_width(Length::Shrink)
                .spacing(5)
                .padding(5)
                .text_size(32),
            self.tabs.get(tab).map(|tab| tab.view(&mut self)),
        ));

        let content = 

        Column::new()
            .push(
                self.tabs.get(self.active_tab)
                .into())
            .push(child)
            .into()
    }
}