use std::sync::Arc;

use fds_toolbox_core::formats::simulations::GlobalTimeSeriesIdx;
use iced::{Command, Element};

use crate::{tabs::Tab, Simulations};

use super::{
    plot::{ChartMessage, Plot2D},
};

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
        let str = "Plot 2D".to_string();
        // for idx in &self.chart.idx {
        //     str.push_str(&idx.to_string());
        // }
        str
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
