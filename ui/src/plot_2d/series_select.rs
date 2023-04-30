use std::collections::HashSet;

use fds_toolbox_lazy_data::sims::{Simulations, BySimulation};
use iced::{Element, widget::{column, text}};
use plotters::style::{Color, ShapeStyle};

pub struct LabeledSeries {
    name: String,
    // color: Box<dyn Into<ShapeStyle>>,
    data: Box<dyn Iterator<Item = (f32, f32)>>,
}

pub trait SeriesSource {
    fn iter_series(&self) -> Box<dyn Iterator<Item = LabeledSeries>>;
}

struct SeriesSelect {
    // series: Vec<LabeledSeries>,
    selected: HashSet<usize>,
}

enum SeriesMessage {
    Select(usize),
    Deselect(usize),
    SelectAll,
    DeselectAll,
}

impl SeriesSelect {
    pub fn view<'a>(&self, model: &'a Simulations) -> Element<'a, SeriesMessage> {
        let mut thing = iced::widget::column![];
        for BySimulation(id, sim) in model.enumerate_simulations() {
            let sim = sim.get();
            match sim {
                Some(sim) => {
                    let mut thing2 = iced::widget::column![];
                    let series = sim.
                    for series in series.iter_series() {
                    }
                    thing.push(thing2)
                }
                None => {
                    thing = thing.push(text(format!("Loading...")));
                }
            }
        }

        thing
    }
}
