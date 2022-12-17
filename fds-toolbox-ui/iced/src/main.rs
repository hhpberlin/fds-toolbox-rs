// TODO: Re-enable and fix the following clippy lints:
// #![warn(clippy::pedantic)]

// TODO: Remove this and remove dead-code once prototyping is done
#![allow(dead_code)]

use std::fmt::Debug;

use fds_toolbox_core::formats::csv::devc::Devices;

use fds_toolbox_core::formats::simulation::{Simulation, TimeSeriesIdx, SliceSeriesIdx};
use fds_toolbox_core::formats::simulations::{SimulationIdx, Simulations};
use fds_toolbox_core::formats::smoke::dim2::slice::Slice;
use iced::widget::{Column, Container, Text};
use iced::{executor, Application, Command, Element, Length, Settings, Theme};
use iced_aw::{TabBar, TabLabel};
use plot_2d::plot_tab::PlotTab;
use slice::slice_tab::SliceTab;
use tabs::{FdsToolboxTab, FdsToolboxTabMessage, Tab};

pub mod plot_2d;
pub mod plotters;
pub mod slice;

pub mod tabs;

mod array_stats_vis;
mod select_list;

pub fn main() -> iced::Result {
    FdsToolbox::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct FdsToolbox {
    active_tab: usize,
    tabs: Vec<FdsToolboxTab>,
    simulations: Simulations,
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
        self.tabs
            .push(FdsToolboxTab::Plot(PlotTab::new(vec![SimulationIdx(
                0,
                TimeSeriesIdx::Device(
                    self.simulations[0]
                        .devc
                        .get_device_idx_by_name("T_B05")
                        .unwrap(),
                ),
            )])));
        self.tabs
            .push(FdsToolboxTab::Plot(PlotTab::new(vec![SimulationIdx(
                0,
                TimeSeriesIdx::Device(
                    self.simulations[0]
                        .devc
                        .get_device_idx_by_name("AST_1OG_Glaswand_N2")
                        .unwrap(),
                ),
            )])));
        self.tabs.push(FdsToolboxTab::Plot(PlotTab::new(
            self.simulations[0]
                .devc
                .iter_device_named_ids()
                .map(|(_, idx)| SimulationIdx(0, TimeSeriesIdx::Device(idx)))
                .collect::<Vec<_>>(),
        )));
        self.tabs.push(FdsToolboxTab::Slice(SliceTab::new(SimulationIdx(
            0,
            SliceSeriesIdx(0),
        ))));
    }
}

impl Application for FdsToolbox {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut this = FdsToolbox {
            active_tab: 0,
            tabs: vec![],
            simulations: Simulations::new(vec![Simulation {
                // TODO: Prompt for files, this is all for testing
                devc: Devices::from_reader(
                    include_bytes!("../../../demo-house/DemoHaus2_devc.csv").as_ref(),
                )
                .unwrap(),
                slcf: vec![Slice::from_reader(
                    include_bytes!("../../../demo-house/DemoHaus2_0004_39.sf").as_ref(),
                )
                .unwrap()],
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
            // We can't actually use self.active_tab() here because of the borrow checker :(
            Message::TabMessage(msg) => match self.tabs.get_mut(self.active_tab) {
                Some(tab) => {
                    return tab
                        .update(&mut self.simulations, msg)
                        .map(Message::TabMessage)
                }
                None => panic!("No active tab"), // TODO: Log error instead of panicking
            },
            // _ => {},
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
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
                    // Column::new(),
                    // |column, tab| {
                    //     let tab_label = <FdsToolboxTab as Tab<Simulations>>::title(tab);
                    //     column.push(Text::new(tab_label))
                    // },
                )
                .on_close(Message::TabClosed)
                .tab_width(Length::Shrink)
                .spacing(5)
                .padding(5)
                .text_size(32)
                .into(),
        };

        let content = match self.tabs.get(self.active_tab) {
            Some(tab) => tab.view(&self.simulations),
            None => Text::new("No tabs open").into(),
        };

        iced::widget::Row::new()
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
