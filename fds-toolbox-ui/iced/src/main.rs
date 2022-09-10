// TODO: Re-enable and fix the following clippy lints:
// #![warn(clippy::pedantic)]

// TODO: Remove this and remove dead-code once prototyping is done
#![allow(dead_code)]

use std::fmt::Debug;

use fds_toolbox_core::formats::csv::devc::Devices;

use fds_toolbox_core::formats::simulation::{Simulation, TimeSeriesIdx};
use fds_toolbox_core::formats::simulations::{GlobalTimeSeriesIdx, Simulations};
use iced::widget::{Column, Text};
use iced::{executor, Application, Command, Container, Element, Length, Row, Settings};
use iced_aw::{TabBar, TabLabel};
use plot_2d::plot_tab::PlotTab;
use tabs::{FdsToolboxTab, FdsToolboxTabMessage, Tab};

pub mod plot_2d;
pub mod tabs;

mod array_stats_vis;
mod select_list;

pub fn main() -> iced::Result {
    FdsToolbox::run(Settings::default())
}

struct FdsToolbox {
    active_tab: usize,
    tabs: Vec<FdsToolboxTab>,
    data: Simulations,
    // TODO: Store using fancy lazy_data structs
    // store: Store,
}

// There will be future messages not relating to tabs, so this is only temporary
// TODO: Add those messages and remove this
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy)]
enum Message {
    TabSelected(usize),
    TabClosed(usize),
    TabMessage(FdsToolboxTabMessage),
}

impl FdsToolbox {
    pub fn active_tab(&mut self) -> Option<&mut FdsToolboxTab> {
        self.tabs.get_mut(self.active_tab)
    }

    fn open_some_tabs(&mut self) {
        self.tabs.push(FdsToolboxTab::Overview(PlotTab::new(vec![
            GlobalTimeSeriesIdx(
                0,
                TimeSeriesIdx::Device(self.data[0].devc.get_device_idx_by_name("T_B05").unwrap()),
            ),
        ])));
        self.tabs.push(FdsToolboxTab::Overview(PlotTab::new(vec![
            GlobalTimeSeriesIdx(
                0,
                TimeSeriesIdx::Device(
                    self.data[0]
                        .devc
                        .get_device_idx_by_name("AST_1OG_Glaswand_N2")
                        .unwrap(),
                ),
            ),
        ])));
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
            data: Simulations::new(vec![Simulation {
                devc: Devices::from_reader(
                    include_bytes!("../../../demo-house/DemoHaus2_devc.csv").as_ref(),
                )
                .unwrap(),
            }]),
        };
        Self::open_some_tabs(&mut this);
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
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let tab_bar: Element<'_, Self::Message> = match self.tabs.len() {
            0 => Column::new().into(),
            _ => self
                .tabs
                .iter()
                .fold(
                    TabBar::new(self.active_tab, Message::TabSelected),
                    |tab_bar, tab| {
                        let tab_label = <FdsToolboxTab as Tab<Simulations>>::title(tab);
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
