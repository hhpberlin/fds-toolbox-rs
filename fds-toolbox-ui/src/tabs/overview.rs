use fds_toolbox_core::formats::csv::devc::Device;
use iced::{Command, Element, Text};

use crate::{FdsToolboxData, plot::{MyChart, ChartMessage}};

use super::Tab;

#[derive(Debug)]
pub struct OverviewTab {
    chart: MyChart,
}

impl OverviewTab {
    pub fn new(device: Device) -> Self {
        Self {
            chart: MyChart::from_(device.iter_f32().collect())
        }
    }
}

impl Tab<FdsToolboxData> for OverviewTab {
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

    fn view<'a, 'b>(&'a mut self, model: &'b FdsToolboxData) -> Element<'_, Self::Message> {
        // let devc = &model.simulations[0].devc;
        // Text::new("Overview").size(20).into()

        // let values = devc.get_device("Abluft_1").unwrap();
        // let mogus: () = MyChart::from_(values);

        self.chart.view()
    }
}
