use iced::{Command, Element};

use crate::{FdsToolboxData, tabs::Tab};

use super::{plot::{ChartMessage, Plot2D}, plottable::Plottable2D};

#[derive(Debug)]
pub struct PlotTab {
    chart: Plot2D,
}

impl PlotTab {
    pub fn new(plt: Box<dyn Plottable2D>) -> Self {
        Self {
            chart: Plot2D::from_single_plottable(plt),
        }
    }
}

impl Tab<FdsToolboxData> for PlotTab {
    type Message = ChartMessage;

    fn title(&self) -> String {
        "Overview".to_string()
    }

    fn update(
        &mut self,
        _model: &mut FdsToolboxData,
        _message: Self::Message,
    ) -> Command<Self::Message> {
        Command::none()
    }

    fn view<'a, 'b>(&'a mut self, _model: &'b FdsToolboxData) -> Element<'_, Self::Message> {
        // let devc = &model.simulations[0].devc;
        // Text::new("Overview").size(20).into()

        // let values = devc.get_device("Abluft_1").unwrap();
        // let mogus: () = MyChart::from_(values);

        self.chart.view()
    }
}
