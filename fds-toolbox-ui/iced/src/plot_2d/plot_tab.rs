use fds_toolbox_core::formats::simulations::GlobalTimeSeriesIdx;
use iced::{Command, Element};

use crate::{tabs::Tab, Simulations};

use super::plot::{ChartMessage, Plot2D};

#[derive(Debug)]
pub struct PlotTab {
    chart: Plot2D<GlobalTimeSeriesIdx>,
}

impl PlotTab {
    pub fn new(idx: Vec<GlobalTimeSeriesIdx>) -> Self {
        Self {
            chart: Plot2D::new(idx),
        }
    }
}

impl Tab<Simulations> for PlotTab {
    type Message = ChartMessage;

    fn title(&self) -> String {
        // TODO: Give a more descriptive name
        //       Maybe list the names of the selected time series?
        // Sub-TODO: Ellispisize long names? Here or generally?
        "Plot 2D".to_string()
    }

    fn update(
        &mut self,
        _model: &mut Simulations,
        _message: Self::Message,
    ) -> Command<Self::Message> {
        Command::none()
    }

    fn view<'a>(&'a mut self, model: &'a Simulations) -> Element<'a, Self::Message> {
        self.chart.view(model)
    }
}
