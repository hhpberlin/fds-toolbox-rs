// TODO: Re-enable and fix
// #![warn(clippy::pedantic)]

// #![warn(clippy::nursery)]
// #![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::correctness)]
#![warn(clippy::perf)]
#![warn(clippy::style)]
#![warn(clippy::suspicious)]
#![warn(clippy::print_stdout)]
#![warn(clippy::print_stderr)]
// #![warn(clippy::todo)]
// #![warn(clippy::unimplemented)]
// #![warn(clippy::dbg_macro)]
// #![warn(clippy::unreachable)]
// #![warn(clippy::panic)]

// #![warn(clippy::unwrap_used)]
// #![warn(clippy::expect_used)]

// TODO: Remove this and remove dead-code once prototyping is done
#![allow(dead_code)]

use std::fmt::Debug;

use fds_toolbox_core::formats::csv::devc::DeviceList;

use fds_toolbox_core::formats::smoke::dim2::slice::Slice;
use fds_toolbox_lazy_data::fs::AnyFs;
use fds_toolbox_lazy_data::moka::{MokaStore};
use fds_toolbox_lazy_data::sims::Simulations;
use iced::event::Status;

use iced::widget::{Column, Container, Text};
use iced::{
    executor, keyboard, subscription, Application, Command, Element, Event, Length, Settings,
    Subscription, Theme,
};
use iced_aw::{TabBar, TabLabel};
use tabs::{FdsToolboxTab, FdsToolboxTabMessage, Tab};

pub mod plot_2d;
pub mod plotters;
pub mod slice;

pub mod tabs;

mod array_stats_vis;
mod select_list;

/// # Errors
///
/// Errors if UI fails to start
pub fn main() -> iced::Result {
    FdsToolbox::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Debug)]
struct FdsToolbox {
    active_tab: usize,
    tabs: Vec<FdsToolboxTab>,
    keyboard_info: KeyboardInfo,
    simulations: Simulations<AnyFs>,
    // TODO: Store using fancy lazy_data structs
    // store: Store,
    // moka_store: MokaStore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct KeyboardInfo {
    modifiers: keyboard::Modifiers,
}

// There will be future messages not relating to tabs, so this is only temporary
// TODO: Add those messages and remove this
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy)]
enum Message {
    TabSelected(TabIdx),
    TabClosed(TabIdx),
    TabMessage(FdsToolboxTabMessage),
}

#[derive(Debug, Clone, Copy)]
enum TabIdx {
    Absolute(usize),
    RelativeToActive(isize),
}

impl TabIdx {
    fn to_absolute_core(self, active: usize, len: usize) -> usize {
        match self {
            TabIdx::Absolute(idx) => idx,
            TabIdx::RelativeToActive(offset) => {
                (active as isize + offset).rem_euclid(len as isize) as usize
            }
        }
    }

    pub fn to_absolute(self, tbx: &FdsToolbox) -> usize {
        self.to_absolute_core(tbx.active_tab, tbx.tabs.len())
    }
}

impl FdsToolbox {
    pub fn active_tab(&mut self) -> Option<&mut FdsToolboxTab> {
        self.tabs.get_mut(self.active_tab)
    }

    fn open_some_tabs(&mut self) {
        // self.tabs
        //     .push(FdsToolboxTab::Plot(PlotTab::new(vec![SimulationIdx(
        //         0,
        //         TimeSeriesIdx::Device(
        //             self.simulations[0]
        //                 .devc
        //                 .get_device_idx_by_name("T_B05")
        //                 .unwrap(),
        //         ),
        //     )])));
        // self.tabs
        //     .push(FdsToolboxTab::Plot(PlotTab::new(vec![SimulationIdx(
        //         0,
        //         TimeSeriesIdx::Device(
        //             self.simulations[0]
        //                 .devc
        //                 .get_device_idx_by_name("AST_1OG_Glaswand_N2")
        //                 .unwrap(),
        //         ),
        //     )])));
        // self.tabs.push(FdsToolboxTab::Plot(PlotTab::new(
        //     self.simulations[0]
        //         .devc
        //         .iter_device_named_ids()
        //         .map(|(_, idx)| SimulationIdx(0, TimeSeriesIdx::Device(idx)))
        //         .collect::<Vec<_>>(),
        // )));
        // self.tabs
        //     .push(FdsToolboxTab::Slice(SliceTab::new(SimulationIdx(
        //         0,
        //         SliceSeriesIdx(0),
        //     ))));
    }

    fn subscription(event: Event, status: Status) -> Option<Message> {
        if let Status::Captured = status {
            return None;
        }

        dbg!(&event);
        match event {
            // Event::Mouse(mouse_event) => match mouse_event {
            //     mouse::Event::ButtonPressed(mouse::Button::Left) => Some(Message::MouseClick),
            //     mouse::Event::CursorMoved { position } => Some(Message::MouseMove(position)),
            //     _ => None,
            // },
            Event::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Tab,
                    modifiers,
                } => {
                    // TODO: Find out why bitflags didn't like matching on the modifiers and then use match instead if possible
                    if modifiers == keyboard::Modifiers::CTRL {
                        Some(Message::TabSelected(TabIdx::RelativeToActive(1)))
                    } else if modifiers == keyboard::Modifiers::SHIFT | keyboard::Modifiers::CTRL {
                        Some(Message::TabSelected(TabIdx::RelativeToActive(-1)))
                    } else {
                        None
                    }
                }
                keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::W,
                    modifiers: keyboard::Modifiers::CTRL,
                } => Some(Message::TabClosed(TabIdx::RelativeToActive(0))),
                _ => None,
            },
            _ => None,
        }
    }
}

impl Application for FdsToolbox {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        // let simulations = Simulations::new(vec![Simulation {
        //     // TODO: Prompt for files, this is all for testing
        //     devc: DeviceList::from_reader(
        //         include_bytes!("../../demo-house/DemoHaus2_devc.csv").as_ref(),
        //     )
        //     .unwrap(),
        //     slcf: vec![Slice::from_reader(
        //         include_bytes!("../../demo-house/DemoHaus2_0001_21.sf").as_ref(),
        //     )
        //     .unwrap()],
        // }]);

        // let moka_store = MokaStore::new(10_000); // TODO: Read this capacity from a config file?

        let mut this = FdsToolbox {
            active_tab: 0,
            tabs: vec![],
            // simulations,
            // moka_store,
            keyboard_info: KeyboardInfo::default(),
            simulations: Default::default(),
        };
        Self::open_some_tabs(&mut this);
        (this, Command::none())
    }

    fn title(&self) -> String {
        "FDS Toolbox".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::TabSelected(idx) => self.active_tab = idx.to_absolute(self),
            Message::TabClosed(idx) => {
                self.tabs.remove(idx.to_absolute(self));
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
                    TabBar::new(self.active_tab, |x| {
                        Message::TabSelected(TabIdx::Absolute(x))
                    }),
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
                .on_close(|x| Message::TabSelected(TabIdx::Absolute(x)))
                .tab_width(Length::Shrink)
                .spacing(5.)
                .padding(5.)
                .text_size(32.)
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

    fn subscription(&self) -> Subscription<Self::Message> {
        subscription::events_with(Self::subscription)
    }
}
