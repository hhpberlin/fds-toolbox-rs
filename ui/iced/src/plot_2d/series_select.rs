use std::{collections::HashSet, fmt::Debug, sync::Arc};

use fds_toolbox_core::common::series::TimeSeries0View;
use fds_toolbox_lazy_data::moka::{MokaStore, SimulationDataError, SimulationIdx};
use iced::{
    widget::{button, checkbox, scrollable, text},
    Command, Element,
};

use crate::old::{plotters::ids::SeriesSourceLine, Model};

#[derive(Debug)]
pub struct SeriesSelection {
    selected: HashSet<Series>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Select(Series),
    Deselect(Series),
    Loaded(Series, Option<Arc<SimulationDataError>>),
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
    fn view<T>(
        &self,
        model: &Model,
        f: impl for<'a> FnOnce(TimeSeries0View<'a>) -> T,
    ) -> Option<T> {
        let Series(sim_idx, series) = *self;
        match series {
            SimSeries::Device { idx } => match model.store.devc().try_get(sim_idx, ()) {
                Some(d) => d.view_device_by_idx(idx).map(f),
                None => None,
            },
        }
    }
}

impl<'a> SeriesSourceLine for (&'a SeriesSelection, &'a Model) {
    fn for_each_series(&self, f: &mut dyn for<'view> FnMut(TimeSeries0View<'view>)) {
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
        let mut thing = iced::widget::column![];
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
            thing = thing.push(Self::thing(
                model.store.sim().try_get(*sim_idx, ()),
                |sim| {
                    iced::widget::column![
                        button(text(format!("Simulation {}", sim.smv.chid))),
                        Self::thing(model.store.devc().try_get(*sim_idx, ()), |devc| {
                            let mut thing2 = iced::widget::column![];
                            thing2 = thing2.push(button("Devices"));
                            for (idx, devc) in devc.iter_device_views().enumerate() {
                                let s = SimSeries::Device { idx };
                                let s = Series(*sim_idx, s);
                                thing2 = thing2.push(checkbox(
                                    devc.name,
                                    self.selected.contains(&s),
                                    move |selected| {
                                        if selected {
                                            Message::Select(s)
                                        } else {
                                            Message::Deselect(s)
                                        }
                                    },
                                ))
                            }
                            thing2.into()
                        }),
                    ]
                    .into()
                },
            ));
        }

        scrollable(thing).into()
    }

    fn thing<'a, T>(
        s: Option<T>,
        render: impl FnOnce(T) -> iced::Element<'a, Message>,
    ) -> iced::Element<'a, Message> {
        match s {
            Some(s) => render(s),
            None => button("Loading...").into(),
        }
    }

    // fn thing<'a, T, E: Debug>(
    //     s: Option<Result<T, E>>,
    //     render: impl FnOnce(T) -> iced::Element<'a, Message>,
    // ) -> iced::Element<'a, Message> {
    //     match s {
    //         Some(Ok(s)) => render(s),
    //         Some(Err(e)) => button(text(format!("{:?}", e))).into(),
    //         None => button("Loading...").into(),
    //     }
    // }

    pub fn update(&mut self, message: Message, model: &Model) -> Command<Message> {
        async fn load(series: Series, model: MokaStore) -> Message {
            let Series(sim_idx, sim_series) = series;
            let result = match sim_series {
                SimSeries::Device { idx: _ } => model.devc().load(sim_idx, ()).await,
            };
            Message::Loaded(series, result.err())
        }

        match message {
            Message::Select(s) => {
                self.selected.insert(s);
                return Command::perform(load(s, model.store.clone()), |x| x);
            }
            Message::Deselect(s) => {
                self.selected.remove(&s);
            }
            Message::Loaded(_, _) => {}
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
