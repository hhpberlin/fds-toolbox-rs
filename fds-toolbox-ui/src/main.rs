#![warn(clippy::pedantic)]

use std::collections::HashMap;

use fds_toolbox_core::formats::Simulation;
use fds_toolbox_core::formats::arr_meta::ArrayStats;
use fds_toolbox_core::formats::csv::devc::Devices;
use iced::pure::Pure;
use iced::widget::{Column, Text};
use iced::{executor, Application, Command, Container, Element, Length, Row, Settings, Scrollable, scrollable, Button, button, pure};
use iced_aw::{TabBar, TabLabel};
use tabs::{FdsToolboxTab, FdsToolboxTabMessage, Tab};

mod panes;
mod tabs;

pub fn main() -> iced::Result {
    FdsToolbox::run(Settings::default())
}

struct FdsToolbox {
    active_tab: usize,
    tabs: Vec<FdsToolboxTab>,
    data: FdsToolboxData,
    sidebar: Sidebar,
}

struct Sidebar {
    state: pure::State,
}

impl Sidebar {
    fn new() -> Self {
        Self {
            state: pure::State::new(),
        }
    }
}

#[derive(Debug)]
struct FdsToolboxData {
    simulations: Vec<Simulation>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    TabSelected(usize),
    TabClosed(usize),
    TabMessage(FdsToolboxTabMessage),
    SidebarMessage(SidebarMessage),
}

impl FdsToolbox {
    pub fn active_tab(&mut self) -> Option<&mut FdsToolboxTab> {
        self.tabs.get_mut(self.active_tab)
    }
}

impl Application for FdsToolbox {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            FdsToolbox {
                active_tab: 0,
                tabs: Vec::new(),
                data: FdsToolboxData {
                    simulations: vec![
                        Simulation {
                            devc: Devices::from_reader(include_bytes!("../../demo-house/DemoHaus2_devc.csv").as_ref()).unwrap(),
                        }
                    ]
                },
                sidebar: Sidebar::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "FDS Toolbox".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::TabSelected(_) => todo!(),
            Message::TabClosed(_) => todo!(),
            Message::TabMessage(_) => todo!(),
            Message::SidebarMessage(message) => match message {
                SidebarMessage::DevcSelected => todo!(),
                SidebarMessage::Scroll(scroll) =>(),// self.sidebar.scroll.snap_to(scroll),
            },
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let sidebar = self.sidebar.view_sidebar(&self.data);

        let tab_bar: Element<'_, Self::Message> = match self.tabs.len() {
            0 => Column::new().into(),
            _ => self
                .tabs
                .iter()
                .fold(
                    TabBar::new(self.active_tab, Message::TabSelected),
                    |tab_bar, tab| {
                        let tab_label = <FdsToolboxTab as Tab<FdsToolboxData>>::title(tab);
                        tab_bar.push(TabLabel::Text(tab_label))
                    },
                )
                .on_close(Message::TabClosed)
                .tab_width(Length::Shrink)
                .spacing(5)
                .padding(5)
                .text_size(32)
                .into(),
        };

        let content = match self.tabs.get_mut(self.active_tab) {
            Some(tab) => tab.view(&self.data),
            None => Text::new("No tabs open").into(),
        };

        Row::new()
            .push(sidebar.map(Message::SidebarMessage))
            .push(
                Column::new().push(tab_bar).push(
                    Container::new(content.map(Message::TabMessage))
                        .width(Length::Fill)
                        .height(Length::Fill),
                ),
            )
            .into()
    }
}

#[derive(Debug)]
struct SidebarBlock<'a, Iter: Iterator, Id> {
    title: &'a str,
    id: Id,
    content: Iter,
}

#[derive(Debug)]
struct DevcSidebar<'a> {
    name: &'a str,
    meta: &'a ArrayStats<f32>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum SidebarId {
    Devc,

}

#[derive(Debug, Clone, Copy)]
enum SidebarMessage {
    DevcSelected,
    Scroll(f32),
}

impl Sidebar {
    // fn add_tab(&mut self, tab: FdsToolboxTab) {
    //     self.tabs.push(tab);
    //     self.active_tab = self.tabs.len() - 1;
    // }

    fn sidebar_content<'a>(data: &'a FdsToolboxData) -> impl Iterator<Item = SidebarBlock<'a, impl Iterator<Item = DevcSidebar<'a>> + 'a, SidebarId>> + 'a {
        let devc = data
            .simulations
            .iter()
            .flat_map(|sim| sim.devc.devices.iter())
            .map(|devc| DevcSidebar {
                name: &devc.name,
                meta: &devc.meta,
            });

        vec![SidebarBlock {
            title: "DEVC",
            id: SidebarId::Devc,
            content: devc,
        }]
        .into_iter()
    }

    fn view_sidebar<'a>(&'a mut self, data: &'a FdsToolboxData) -> Element<'a, SidebarMessage> {
        Pure::new(&mut self.state, Self::view_sidebar_pure(data)).into()
    }

    fn view_sidebar_pure(data: &FdsToolboxData) -> pure::Element<'_, SidebarMessage> {
        let mut col = pure::column();

        for block in Self::sidebar_content(data) {
            let mut content = pure::column()
                .push(pure::button(pure::text(block.title).size(20)).on_press(SidebarMessage::DevcSelected))
                // .spacing(5)
                .padding(10);

            for elem in block.content {
                content = content
                    .push(pure::button(pure::text(elem.name).size(12)).on_press(SidebarMessage::DevcSelected));
            }

            col = col.push(content);
        }

        pure::scrollable(col).into()
    }
}