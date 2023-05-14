use fds_toolbox_lazy_data::moka::{SimulationDataIdx, SimulationIdx, SimulationsDataIdx};
use iced::{
    widget::{button, text},
    Command, Element,
};
use tracing::error;

use crate::old::{tabs::Tab, Message, Model};

#[derive(Debug)]
struct OverviewTab(SimulationIdx);

impl Tab for OverviewTab {
    type Message = super::Message;

    fn title(&self) -> String {
        "Overview".to_string()
    }

    fn update(&mut self, _model: &mut Model, _message: Self::Message) -> Command<Self::Message> {
        // match message {}

        Command::none()
    }

    fn view<'a>(&'a self, model: &'a Model) -> Element<'a, Self::Message> {
        //model.active_simulations

        iced::widget::row(
            model
                .active_simulations
                .iter()
                .map(|sim| {
                    button(text(match model.store.get_path_by_idx(*sim) {
                        Some(path) => path.chid,
                        None => {
                            error!("No path for simulation {:?}", sim);
                            "Error".to_string()
                        }
                    }))
                    .on_press(Message::Load(SimulationsDataIdx(
                        *sim,
                        SimulationDataIdx::Simulation,
                    )))
                    .into()
                })
                .collect::<Vec<_>>(),
        )
        .into()
    }
}
