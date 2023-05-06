use std::{collections::HashSet, fmt::Debug, sync::Arc};

use fds_toolbox_core::{common::series::TimeSeries0View, formats::csv::devc::DeviceList};
use fds_toolbox_lazy_data::moka::{DeviceIdx, SimulationIdx};
use iced::{
    futures::FutureExt,
    widget::{button, checkbox, scrollable, text},
    Command, Element,
};
use ndarray::{Ix0, Ix1};

use crate::plotters::ids::SeriesSource;
use crate::{plotters::ids::Viewable, Model};
// pub struct LabeledSeries {
//     name: String,
//     // color: Box<dyn Into<ShapeStyle>>,
//     data: Box<dyn Iterator<Item = (f32, f32)>>,
// }

#[derive(Debug)]
pub struct SeriesSelection {
    // series: Vec<LabeledSeries>,
    selected: HashSet<Series>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Select(Series),
    Deselect(Series),
    SelectAll,
    DeselectAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Series(SimulationIdx, SimSeries);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimSeries {
    Device { idx: usize },
    // Slice { sim: SimulationIdx, idx: usize, x: usize, y: usize },
}

impl Series {
    fn view<T>(&self, model: &Model, f: impl FnOnce(TimeSeries0View) -> T) -> Option<T> {
        let Series(sim_idx, series) = self;
        match *series {
            SimSeries::Device { idx } => match model.store.get_devc(*sim_idx).now_or_never() {
                Some(Ok(d)) => d.view_device_by_idx(idx).map(f),
                _ => None,
            },
        }
    }
}

impl SeriesSource<Ix1> for (&SeriesSelection, &Model) {
    fn for_each_series(&self, f: &mut dyn FnMut(TimeSeries0View)) {
        let (selection, model) = *self;
        for series in &selection.selected {
            series.view(model, &mut *f);
        }
    }
}

impl SeriesSelection {
    pub fn new() -> Self {
        Self {
            // series: vec![],
            selected: HashSet::new(),
        }
    }

    pub fn view<'a>(&self, model: &'a Model) -> Element<'a, Message> {
        let thing = iced::widget::column![];
        // for BySimulation(id, sim) in model.enumerate_simulations() {
        //     let sim = sim.get().and_then(CacheResult::into_val);
        //     match sim {
        //         Some(sim) => {
        //             if let Some(Ok(series)) = sim.get_devc().try_get() {
        //                 let mut thing2 = iced::widget::column![];
        //                 for series in series.iter_device_views() {
        //                     thing2.push(checkbox(
        //                         format!("{}", series.name,),
        //                         false,
        //                         move |selected| {
        //                             if selected {
        //                                 SeriesMessage::Select(0)
        //                             } else {
        //                                 SeriesMessage::Deselect(0)
        //                             }
        //                         },
        //                     ));
        //                 }
        //                 thing.push(thing2);
        //             } else {
        //                 thing = thing.push(text(format!("Loading...")));
        //             }
        //         }
        //         _ => {
        //             thing = thing.push(text(format!("Loading...")));
        //         }
        //     }
        // }

        for sim_idx in &model.active_simulations {
            Self::thing(model.store.get_sim(*sim_idx).now_or_never(), |sim| {
                iced::widget::column![
                    button(text(format!("Simulation {}", sim.smv.chid))),
                    Self::thing(model.store.get_devc(*sim_idx).now_or_never(), |devc| {
                        let mut thing2 = iced::widget::column![];
                        thing2 = thing2.push(button("Devices"));
                        for (idx, devc) in devc.iter_device_views().enumerate() {
                            thing2 = thing2.push(checkbox(devc.name, false, move |selected| {
                                if selected {
                                    Message::Select(Series(*sim_idx, SimSeries::Device { idx }))
                                } else {
                                    Message::Deselect(Series(*sim_idx, SimSeries::Device { idx }))
                                }
                            }))
                        }
                        thing2.into()
                    }),
                ]
                .into()
            });
        }

        scrollable(thing).into()
    }

    fn thing<'a, T, E: Debug>(
        s: Option<Result<T, E>>,
        render: impl FnOnce(T) -> iced::Element<'a, Message>,
    ) -> iced::Element<'a, Message> {
        match s {
            Some(Ok(s)) => render(s),
            Some(Err(e)) => button(text(format!("{:?}", e))).into(),
            None => button("Loading...").into(),
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Select(s) => {
                self.selected.insert(s);
            }
            Message::Deselect(s) => {
                self.selected.remove(&s);
            }
            Message::SelectAll => todo!(),
            Message::DeselectAll => {
                self.selected.clear();
            }
        }
        Command::none()
    }
}

impl Default for SeriesSelection {
    fn default() -> Self {
        Self::new()
    }
}
