use std::{borrow::Cow, collections::HashSet, sync::Arc};

use fds_toolbox_core::{
    file::{OsFs, SimulationPath, SliceSeriesIdx},
    formats::csv::devc::{DeviceIdx, DeviceReadings},
};
use fds_toolbox_lazy_data::{
    fs::AnyFs,
    moka::{
        MokaStore, P3dIdx, S3dIdx, SimulationData, SimulationDataError, SimulationDataIdx,
        SimulationIdx, SimulationsDataIdx,
    },
};
use iced::{
    executor,
    widget::{button, column, container, pick_list, row, scrollable, text},
    Alignment, Application, Command, Element, Renderer, Theme,
};
use iced_aw::Grid;
use tracing::{debug, error, info};

// use crate::sidebar::{self, Dummy, Group, Quantity, Series0, Series2, Series3, Series3Type, Series2Type, Series0Type, SelectionSrc};

#[derive(Debug)]
pub struct FdsToolbox {
    pub active_simulations: Vec<SimulationIdx>,
    pub store: MokaStore,
    pub tabs: Vec<Tab>,
    active_tab: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Used for when returning a message is required but not desired.
    NoOp,
    OpenSimulationFileDialog,
    OpenSimulation(SimulationPath<AnyFs>),
    Unload(SimulationsDataIdx),
    Unloaded(SimulationsDataIdx),
    Load(SimulationsDataIdx),
    Loaded(Result<SimulationData, Arc<SimulationDataError>>),
    TabChanged(usize),
    // This could omit the usize and just reference active_tab, but that could cause problems
    // if the active_tab is changed before the message is processed
    // (such as if Command::perform is used with some long-running future).
    TabMessage(usize, TabMessage),
    TabOpen(Tab),
}

#[derive(Debug, Clone)]
pub enum TabMessage {
    Replace(Tab),
}

#[derive(Debug, Clone)]
pub enum Tab {
    HomeTab,
    Overview(SimulationIdx),
}

impl Application for FdsToolbox {
    type Message = Message;
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut this = Self {
            active_simulations: vec![],
            store: MokaStore::new(100_000),
            tabs: vec![Tab::HomeTab],
            active_tab: 0,
        };
        let path = SimulationPath::new(
            AnyFs::LocalFs(OsFs),
            "demo-house".to_string(),
            "DemoHaus2.smv",
        );
        let idx = this.store.get_idx_by_path(&path);
        this.active_simulations.push(idx);
        let store = this.store.clone();
        (
            this,
            Command::perform(
                async move {
                    Message::Loaded(
                        store
                            .get(SimulationsDataIdx(idx, SimulationDataIdx::DevciceList))
                            .await,
                    )
                },
                |x| x,
            ),
        )
        // (this, Command::none())
    }

    fn title(&self) -> String {
        "FDS Toolbox".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Load(idx) => {
                let store = self.store.clone();
                return Command::perform(
                    async move { Message::Loaded(store.get(idx).await) },
                    |x| x,
                );
            }
            Message::Unload(idx) => {
                let store = self.store.clone();
                return Command::perform(
                    async move {
                        store.unload(idx).await;
                        Message::Unloaded(idx)
                    },
                    |x| x,
                );
            }
            Message::Unloaded(idx) => debug!("Unloaded simulation data {:?}", idx),
            Message::Loaded(Ok(data)) => {
                debug!("Loaded simulation data {:?}", data);
            }
            Message::Loaded(Err(err)) => error!("Error loading simulation data: {:?}", err),
            Message::OpenSimulationFileDialog => {
                return Command::perform(
                    async {
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter("Smokeview", &["smv"])
                            .pick_file()
                            .await;
                        let Some(file) = file else {
                            return Message::NoOp;
                        };
                        let path = file.path();
                        let Some(dir) = path.parent() else {
                                error!("Could not get parent directory of file {:?}", path);
                         return Message::NoOp;
                        };
                        let Some((path, dir)) = path.to_str().zip(dir.to_str()) else {
                            error!("Could not convert path to string: {:?}", path);
                            return Message::NoOp;
                        };
                        let (path, dir) = (path.to_string(), dir.to_string());

                        let sim_path = SimulationPath::new_full(AnyFs::LocalFs(OsFs), dir, path);
                        Message::OpenSimulation(sim_path)
                    },
                    |x| x,
                );
            }
            Message::OpenSimulation(path) => {
                let idx = self.store.get_idx_by_path(&path);
                self.active_simulations.push(idx);
                debug!("Added simulation {:?} with idx {:?}", path, idx);

                // NOTE: This is technically not required, but it's just about always wanted.
                return Command::perform(
                    async move { Message::Load(SimulationsDataIdx(idx, SimulationDataIdx::Simulation)) },
                    |x| x,
                );
            }
            Message::NoOp => {}
            Message::TabChanged(idx) => {
                self.active_tab = idx;
            }
            Message::TabMessage(idx, msg) => match msg {
                TabMessage::Replace(tab) => {
                    self.tabs[idx] = tab;
                }
            },
            Message::TabOpen(tab) => self.tabs.push(tab),
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        // let grid = Grid::with_columns(2);

        // dbg!(sidebar::simulation(
        //     &self.store,
        //     Dummy,
        //     *self.active_simulations.first().unwrap()
        // ));
        // scrollable(
        //     sidebar::simulation(
        //         &self.store,
        //         Dummy,
        //         *self.active_simulations.first().unwrap(),
        //     )
        //     .view(),
        // )
        let core: Element<_> = match self.tabs[self.active_tab] {
            Tab::HomeTab => {
                debug!("{:?}", self.active_simulations);
                let sim: Element<_> = match self.active_simulations.is_empty() {
                    true => text("No simulations loaded.").into(),
                    false => iced::widget::column(
                        self.active_simulations
                            .iter()
                            .map(|&x| {
                                button(text(self.try_get_name_infallible(x)))
                                    .on_press(Message::TabOpen(Tab::Overview(x)))
                                    .into()
                            })
                            .collect(),
                    )
                    .into(),
                };
                container(column!(
                    sim,
                    button("Open simulation").on_press(Message::OpenSimulationFileDialog),
                ))
                .center_x()
                .center_y()
                // .align_x(Alignment::Center)
                // .align_y(Alignment::Center)
                .into()
            }
            Tab::Overview(sim_idx) => {
                let sim_selection = pick_list(
                    Cow::Owned(
                        self.active_simulations
                            .iter()
                            .map(|&x| KeyedStr(x, self.try_get_name_infallible(x)))
                            .collect(),
                    ),
                    Some(KeyedStr(sim_idx, "".to_string())),
                    |x| self.tab_msg(TabMessage::Replace(Tab::Overview(x.0))),
                );

                container(column!(
                    sim_selection,
                    // scrollable(
                    //     sidebar::simulation(
                    //         &self.store,
                    //         Dummy,
                    //         *self.active_simulations.first().unwrap(),
                    //     )
                    //     .view(),
                    // ),
                ))
                .center_x()
                .center_y()
                .into()
            }
        };

        row!(core, text(format!("Sims: {:?}", self.active_simulations))).into()
    }
}

#[derive(Debug, Clone)]
struct KeyedStr<Key>(Key, String);
impl<Key> ToString for KeyedStr<Key> {
    fn to_string(&self) -> String {
        self.1.clone()
    }
}
impl<Key: Eq> Eq for KeyedStr<Key> {}
impl<Key: PartialEq> PartialEq for KeyedStr<Key> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl FdsToolbox {
    fn try_get_name(&self, idx: SimulationIdx) -> Option<String> {
        match self.store.sim().try_get(idx, ()) {
            Some(x) => Some(x.smv.chid.to_owned()),
            None => match self.store.get_path_by_idx(idx) {
                Some(x) => Some(x.smv),
                None => None,
            },
        }
    }

    fn try_get_name_infallible(&self, idx: SimulationIdx) -> String {
        self.try_get_name(idx).unwrap_or_else(|| {
            error!("Could not get name for simulation {:?}", idx);
            "Unloaded simulation (error)".to_string()
        })
    }

    fn tab_msg(&self, msg: TabMessage) -> Message {
        Message::TabMessage(self.active_tab, msg)
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum Quantity {
//     Temperature,
//     VolumeFlowRate,
//     ReciprocalLength,
//     // Other(&'static str),
// }

// impl Quantity {
//     pub fn from_str(s: &str) -> Option<Self> {
//         match s {
//             "C" => Some(Quantity::Temperature),
//             "m3/s" => Some(Quantity::VolumeFlowRate),
//             "1/m" => Some(Quantity::ReciprocalLength),
//             _ => None,
//         }
//     }

//     pub fn to_str(self) -> &'static str {
//         match self {
//             Quantity::Temperature => "C",
//             Quantity::VolumeFlowRate => "m3/s",
//             Quantity::ReciprocalLength => "1/m",
//         }
//     }
// }

// fn list_devc(store: &MokaStore, sim: SimulationIdx) -> impl Iterator<Item = DeviceIdx> {

// }

// fn view_s0_sel<F: Fn(bool) -> Msg, Msg>(store: &MokaStore, selection: impl Fn(Series0) -> (bool, F)) -> iced::Element<'_, Msg> {

// }

trait TableElement<'a, Message> {
    fn view_col(&mut self, column: usize) -> iced::Element<'a, Message>;
    fn columns() -> &'static [&'static str];
    fn compare_by_column(&self, other: &Self, column: usize) -> std::cmp::Ordering;
}

type TableOrdering = Option<(usize, SortingDirection)>;
#[derive(Debug, Clone, Copy)]
enum SortingDirection {
    Ascending,
    Descending,
}

fn table<'a, Msg: Clone + 'a + 'static, T: TableElement<'a, Msg>>(
    iter: &mut [T],
    ordering: TableOrdering,
    ordering_msg: impl Fn(usize) -> Msg,
) -> iced::Element<'a, Msg> {
    if let Some((col, direction)) = ordering {
        iter.sort_by(move |a, b| {
            let ordering = a.compare_by_column(b, col);
            match direction {
                SortingDirection::Ascending => ordering,
                SortingDirection::Descending => ordering.reverse(),
            }
        });
    }

    let columns = T::columns().len();

    let mut grid = Grid::with_columns(columns);

    for (i, &col) in T::columns().iter().enumerate() {
        let arrow = match ordering {
            Some((col_i, SortingDirection::Ascending)) if i == col_i => "▲",
            Some((col_i, SortingDirection::Descending)) if i == col_i => "▼",
            _ => "",
        };
        let btn = button(row!(text(arrow), text(col))).on_press(ordering_msg(i));
        grid.insert(btn);
    }

    for row in iter {
        for col in 0..columns {
            grid.insert(row.view_col(col));
        }
    }
    grid.into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Series0Type {
    Device,
    Hrr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Series0 {
    Device(SimulationIdx, DeviceIdx),
    Hrr(SimulationIdx, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Series2Type {
    Slice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Series2 {
    Slice {
        sim: SimulationIdx,
        idx: SliceSeriesIdx,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Series3Type {
    S3D,
    P3D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Series3 {
    S3D(SimulationIdx, S3dIdx),
    P3D(SimulationIdx, P3dIdx),
}
