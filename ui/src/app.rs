use std::sync::Arc;

use fds_toolbox_core::file::{OsFs, SimulationPath};
use fds_toolbox_lazy_data::{
    fs::AnyFs,
    moka::{MokaStore, SimulationData, SimulationDataError, SimulationIdx, SimulationsDataIdx},
};
use iced::{executor, Application, Command, Element, Renderer, Theme};
use iced_aw::Grid;
use tracing::{debug, error, info};

use crate::sidebar::{self, Dummy};

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
                        store.unload(idx.clone()).await;
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

        dbg!(sidebar::simulation(
            &self.store,
            Dummy,
            *self.active_simulations.first().unwrap()
        ));

        todo!()
    }
}
