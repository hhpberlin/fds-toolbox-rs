use std::fmt::Debug;
use std::sync::Arc;

use fds_toolbox_core::file::{OsFs, SimulationPath};
use fds_toolbox_lazy_data::fs::AnyFs;
use fds_toolbox_lazy_data::moka::{
    MokaStore, SimulationData, SimulationDataError, SimulationIdx, SimulationsDataIdx, SimulationDataIdx,
};
// use fds_toolbox_lazy_data::sims::Simulations;
use iced::event::Status;

use iced::widget::{Column, Container, Text};
use iced::{
    executor, keyboard, subscription, Application, Command, Element, Event, Length,
    Subscription, Theme,
};
use iced_aw::{TabBar, TabLabel};
use plot_2d::plot_tab::PlotTab;
use tabs::{FdsToolboxTab, FdsToolboxTabMessage, Tab};
use tracing::{error, info};
// use tracing_subscriber;

pub mod plot_2d;
pub mod plotters;
pub mod slice;

pub mod tabs;

mod array_stats_vis;
mod overview;
mod select_list;

#[derive(Debug)]
pub struct FdsToolbox {
    active_tab: usize,
    tabs: Vec<FdsToolboxTab>,
    keyboard_info: KeyboardInfo,
    // simulations: Simulations,
    // TODO: Store using fancy lazy_data structs
    // store: Store,
    simulations: Simulations,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct KeyboardInfo {
    modifiers: keyboard::Modifiers,
}

// There will be future messages not relating to tabs, so this is only temporary
// TODO: Add those messages and remove this
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(TabIdx),
    TabClosed(TabIdx),
    TabMessage(FdsToolboxTabMessage),
    // LoadedSims(Option<Arc<SimulationDataError>>),
    Load(SimulationsDataIdx),
    Loaded(Result<SimulationData, Arc<SimulationDataError>>),
}

#[derive(Debug, Clone, Copy)]
pub enum TabIdx {
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

pub type Model = Simulations;

#[derive(Debug)]
pub struct Simulations {
    pub store: MokaStore,
    pub active_simulations: Vec<SimulationIdx>,
}

impl Simulations {
    fn new() -> Self {
        Self {
            store: MokaStore::new(1_000_000),
            active_simulations: Vec::new(),
        }
    }
}

impl FdsToolbox {
    pub fn active_tab(&mut self) -> Option<&mut FdsToolboxTab> {
        self.tabs.get_mut(self.active_tab)
    }

    fn open_some_tabs(&mut self) {
        // info!("{:?}", std::fs::read_dir("./demo-house").unwrap().into_iter().collect::<Vec<_>>());
        self.simulations
            .active_simulations
            .push(self.simulations.store.get_idx_by_path(&SimulationPath::new(
                AnyFs::LocalFs(OsFs),
                "./demo-house".to_string(),
                "DemoHaus2".to_string(),
            )));
        self.tabs.push(FdsToolboxTab::Plot(PlotTab::new()));
        // self.tabs.push(FdsToolboxTab::)
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

    // fn load_simulations(&mut self) -> Command<Message> {
    //     Command::batch(self.simulations.active_simulations.iter().map(|&x| {
    //         let store = self.simulations.store.clone();
    //         Command::perform(
    //             async move {
    //                 // Implicitly loads sim as well
    //                 let res = store.devc().get(x, ()).await;
    //                 Message::loade(res.err())
    //             },
    //             |x| x,
    //         )
    //     }))
    // }

    fn load(&self, idx: SimulationsDataIdx) -> Command<Message> {
        let store = self.simulations.store.clone();

        Command::perform(
            async move {
                let res = store.get(idx).await;
                Message::Loaded(res)
            },
            |x| x,
        )
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
            keyboard_info: KeyboardInfo::default(),
            simulations: Simulations::new(),
        };
        Self::open_some_tabs(&mut this);
        let idx = *this.simulations.active_simulations.first().unwrap();
        let load_simulations = Command::perform(async {}, move |_| {
            Message::Load(SimulationsDataIdx(idx, SimulationDataIdx::Simulation))
        });
        // let load_simulations = this.load_simulations();
        (this, load_simulations)
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
            Message::Load(idx) => return self.load(idx),
            Message::Loaded(res) => match res {
                Ok(data) => {
                    info!("Loaded data: {:?}", data);
                }
                Err(err) => error!("{:?}", err),
            },
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
                        let tab_label = <FdsToolboxTab as Tab>::title(tab);
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
