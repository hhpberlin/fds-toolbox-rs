#![warn(clippy::pedantic)]

use fds_toolbox_core::formats::csv::devc::Devices;
use fds_toolbox_core::formats::Simulation;

use iced::widget::{Column, Text};
use iced::{executor, Application, Command, Container, Element, Length, Row, Settings};
use iced_aw::{TabBar, TabLabel};
use plot_2d::plot_tab::PlotTab;
use tabs::{FdsToolboxTab, FdsToolboxTabMessage, Tab};

pub mod plot_2d;
pub mod tabs;

mod array_stats_vis;
mod select_list;
// mod sidebar;

// use sidebar::Sidebar;

pub fn main() -> iced::Result {
    FdsToolbox::run(Settings::default())
}

struct FdsToolbox {
    active_tab: usize,
    tabs: Vec<FdsToolboxTab>,
    data: FdsToolboxData,
    // sidebar: Sidebar,
}

#[derive(Debug)]
pub struct FdsToolboxData {
    simulations: Vec<Simulation>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    TabSelected(usize),
    TabClosed(usize),
    TabMessage(FdsToolboxTabMessage),
    // SidebarMessage(sidebar::SidebarMessage),
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
        let mut this = FdsToolbox {
            active_tab: 0,
            tabs: vec![],
            data: FdsToolboxData {
                simulations: vec![Simulation {
                    devc: Devices::from_reader(
                        include_bytes!("../../demo-house/DemoHaus2_devc.csv").as_ref(),
                    )
                    .unwrap(),
                }],
            },
            // sidebar: Sidebar::new(),
        };
        this.tabs.push(FdsToolboxTab::Overview(PlotTab::new(
            this.data.simulations[0]
                .devc
                .get_device("T_B05")
                .unwrap()
                .iter_f32()
                .collect(),
        )));
        this.tabs.push(FdsToolboxTab::Overview(PlotTab::new(
            this.data.simulations[0]
                .devc
                .get_device("AST_1OG_Glaswand_N2")
                .unwrap()
                .iter_f32()
                .collect(),
        )));
        (this, Command::none())
    }

    fn title(&self) -> String {
        "FDS Toolbox".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::TabSelected(tab) => self.active_tab = tab,
            Message::TabClosed(tab) => {
                self.tabs.remove(tab);
            }
            Message::TabMessage(_) => todo!(),
            // Message::SidebarMessage(message) => match message {
            //     sidebar::SidebarMessage::DevcSelected => todo!(),
            // },
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        // let sidebar = self.sidebar.view_sidebar(&self.data);

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
            // .push(sidebar.map(Message::SidebarMessage))
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
