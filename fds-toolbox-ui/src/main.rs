#![warn(clippy::pedantic)]

use fds_toolbox_core::formats::Simulation;
use fds_toolbox_core::formats::arr_meta::ArrayStats;
use fds_toolbox_core::formats::csv::devc::Devices;
use iced::widget::{Column, Text};
use iced::{executor, Application, Command, Container, Element, Length, Row, Settings, Scrollable, scrollable, Button, button};
use iced_aw::{TabBar, TabLabel};
use tabs::{FdsToolboxTab, FdsToolboxTabMessage, Tab};

mod panes;
mod tabs;

pub fn main() -> iced::Result {
    FdsToolbox::run(Settings::default())
}

#[derive(Debug)]
struct FdsToolbox {
    active_tab: usize,
    tabs: Vec<FdsToolboxTab>,
    data: FdsToolboxData,
    sidebar: Sidebar,
}

#[derive(Debug)]
struct Sidebar {
    scroll: scrollable::State,
    buttons: Vec<button::State>,
}

impl Sidebar {
    fn new() -> Self {
        Self {
            scroll: scrollable::State::new(),
            buttons: vec![button::State::new(); 3],
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
            Some(tab) => tab.view(&mut self.data),
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

struct SidebarBlock<'a, Iter: Iterator> {
    title: &'a str,
    content: Iter,
}

struct DevcSidebar<'a> {
    name: &'a str,
    meta: &'a ArrayStats<f32>,
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

    fn sidebar_content<'a>(data: &'a FdsToolboxData) -> impl Iterator<Item = SidebarBlock<'a, impl Iterator<Item = DevcSidebar<'a>> + 'a>> + 'a {
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
            content: devc,
        }]
        .into_iter()
    }

    fn view_sidebar(&mut self, data: &FdsToolboxData) -> Element<'_, SidebarMessage> {
        let mut col = Scrollable::new(&mut self.scroll)
            .on_scroll(SidebarMessage::Scroll);

        for block in Self::sidebar_content(data) {
            col = col.push(
                Column::new()
                    // .push(Button::new(self.buttons, Text::new(block.title).size(20)))
                    .push(
                        block
                            .content
                            .fold(Column::new(), |col, devc| {
                                col.push(Text::new(devc.name))
                            })
                            .spacing(5)
                            .padding(10),
                    ),
            );
        }

        col.into()
    }
}