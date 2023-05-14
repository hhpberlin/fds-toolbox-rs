use std::sync::Arc;

use fds_toolbox_lazy_data::moka::{
    MokaStore, SimulationDataError, SimulationIdx, SimulationsDataIdx, SimulationData,
};
use iced::{executor, Application, Command, Element, Theme, Renderer};
use tracing::{debug, error, info};

#[derive(Debug)]
pub struct FdsToolbox {
    pub active_simulations: Vec<SimulationIdx>,
    pub store: MokaStore,
}

#[derive(Debug)]
pub enum Message {
    Load(SimulationsDataIdx),
    Loaded(Result<SimulationData, Arc<SimulationDataError>>),
}

impl Application for FdsToolbox {
    type Message = Message;
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let this = Self {
            active_simulations: vec![],
            store: MokaStore::new(100_000),
        };
        (this, Command::none())
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
            Message::Loaded(Ok(data)) => debug!("Loaded simulation data {:?}", data),
            Message::Loaded(Err(err)) => error!("Error loading simulation data: {:?}", err),
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        
    }
}
