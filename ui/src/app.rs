use std::{collections::HashSet, sync::Arc};

use fds_toolbox_core::file::{OsFs, SimulationPath};
use fds_toolbox_lazy_data::{
    fs::AnyFs,
    moka::{MokaStore, SimulationData, SimulationDataError, SimulationIdx, SimulationsDataIdx},
};
use iced::{executor, widget::scrollable, Application, Command, Element, Renderer, Theme};
use iced_aw::Grid;
use tracing::{debug, error, info};

use crate::sidebar::{self, Dummy, Group, Quantity, Series0, Series2, Series3, Series3Type, Series2Type, Series0Type, SelectionSrc};

#[derive(Debug)]
pub struct FdsToolbox {
    pub active_simulations: Vec<SimulationIdx>,
    pub store: MokaStore,
}

#[derive(Debug, Clone)]
pub enum Message {
    Unload(SimulationsDataIdx),
    Unloaded(SimulationsDataIdx),
    Load(SimulationsDataIdx),
    Loaded(Result<SimulationData, Arc<SimulationDataError>>),
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
        };
        let path = SimulationPath::new(
            AnyFs::LocalFs(OsFs),
            "demo-house".to_string(),
            "DemoHaus2".to_string(),
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
                            .get(SimulationsDataIdx(
                                idx,
                                fds_toolbox_lazy_data::moka::SimulationDataIdx::DevciceList,
                            ))
                            .await,
                    )
                },
                |x| x,
            ),
        )
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
            Message::Loaded(Ok(data)) => debug!("Loaded simulation data {:?}", data),
            Message::Loaded(Err(err)) => error!("Error loading simulation data: {:?}", err),
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
        scrollable(
            sidebar::simulation(
                &self.store,
                Dummy,
                *self.active_simulations.first().unwrap(),
            )
            .view(),
        )
    }
}

// TODO: HashSets are overkill for this, these are small enums and could be keyed by their discriminant directly
#[derive(Debug, Default)]
struct Expanded {
    grp: HashSet<Group>,
    s0: HashSet<Series0Type>,
    s0_qty: HashSet<(Series0Type, Quantity)>,
    s2: HashSet<Series2Type>,
    s2_qty: HashSet<(Series2Type, Quantity)>,
    s3: HashSet<Series3Type>,
    s3_qty: HashSet<(Series3Type, Quantity)>,
}

#[derive(Debug, Default)]
struct Selection {
    s0: HashSet<Series0>,
    s2: HashSet<Series2>,
    s3: HashSet<Series3>,
}

#[derive(Debug)]
struct SelSrc {
    sel: Selection,
    exp: Expanded,
    s0_allowed: bool,
    s2_allowed: bool,
    s3_allowed: bool,
}

impl SelSrc {
    pub fn new(s0_allowed: bool, s2_allowed: bool, s3_allowed: bool) -> Self {
        Self {
            sel: Selection::default(),
            exp: Expanded::default(),
            s0_allowed,
            s2_allowed,
            s3_allowed,
        }
    }
}

impl SelectionSrc for SelSrc {
    fn simulation(&self, idx: SimulationIdx) -> SelectionState<Message> {
        todo!()
    }

    fn group(&self, idx: sidebar::SimulationGroup) -> SelectionState<Message> {
        todo!()
    }

    fn group_quantity(&self, idx: sidebar::SimulationGroupQuantity) -> SelectionState<Message> {
        todo!()
    }

    fn series0(&self, idx: Series0) -> SelectionState<Message> {
        todo!()
    }

    fn series2(&self, idx: Series2) -> SelectionState<Message> {
        todo!()
    }

    fn series3(&self, idx: Series3) -> SelectionState<Message> {
        todo!()
    }

    fn load_msg(&self, idx: SimulationsDataIdx) -> SelectionMessages<Message> {
        todo!()
    }
}