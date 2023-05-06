use std::cell::RefCell;

use fds_toolbox_core::common::series::{SeriesView, TimeSeries2Frame};
use fds_toolbox_lazy_data::moka::{SimulationIdx, SliceIdx};
use iced::{futures::FutureExt, widget::row, Command, Element};
use ndarray::Ix2;

use crate::{
    plotters::{
        cartesian::{self, cartesian},
        heatmap::Heatmap,
        ids::SeriesSource,
    },
    tabs::Tab,
    Model,
};

#[derive(Debug)]
pub struct SliceTab {
    slice: (SimulationIdx, SliceIdx, usize),
    frame: usize,
    plot_state: RefCell<cartesian::State>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Plot(cartesian::Message),
    // AddSlice(SliceFile),
}

impl SliceTab {
    pub fn new(slice: (SimulationIdx, SliceIdx, usize)) -> Self {
        Self {
            slice,
            frame: 200, // TODO
            plot_state: RefCell::new(cartesian::State::new(
                (0.0..=10.0).into(),
                (0.0..=10.0).into(),
            )),
        }
    }

    // fn view_sidebar<'a>(
    //     mut series: RefMut<'a, HashMap<SimulationIdx<SliceSeriesIdx>, PlotTabSeries>>,
    //     model: &'a Simulations,
    // ) -> Element<'a, Message> {
    //     let mut sidebar = Column::new();

    //     for (idx, device) in model
    //         .simulations
    //         .iter()
    //         .flat_map(|x| x.devc.enumerate_devices())
    //     {
    //         // TODO: This does not work with multiple simulations
    //         let global_idx = SimulationIdx(0, TimeSeriesIdx::Device(idx));

    //         let info = series
    //             .entry(global_idx)
    //             .or_insert_with(|| PlotTabSeries::new(global_idx));

    //         sidebar = sidebar
    //             .push(row![
    //                 container(checkbox(
    //                     format!("{} ({})", device.name, device.unit),
    //                     info.selected,
    //                     move |checked| {
    //                         if checked {
    //                             Message::Add(global_idx)
    //                         } else {
    //                             Message::Remove(global_idx)
    //                         }
    //                     },
    //                 ))
    //                 .width(Length::Shrink),
    //                 horizontal_space(Length::Fill),
    //                 container(array_stats_vis(device.values.stats))
    //                     .width(Length::Units(100))
    //                     .height(Length::Units(20)),
    //             ])
    //             .max_width(400);
    //     }

    //     scrollable(sidebar).into()
    // }
}

impl SeriesSource for (&SliceTab, &Model) {
    type Item<'a> = TimeSeries2Frame<'a>;

    fn for_each_series(&self, f: &mut dyn for<'a> FnMut(TimeSeries2Frame<'a>))
    {
        let (tab, model) = *self;
        let slice = model
            .store
            .get_slice(tab.slice.0, tab.slice.1)
            .now_or_never();

        let Some(Ok(slice)) = slice else {return;};
        let Some(series) = slice
            .data.view().view_frame(tab.slice.2) else { return;};

        f(series);
    }
}

impl Tab for SliceTab {
    type Message = Message;

    fn title(&self) -> String {
        "Slice Plot".to_string()
    }

    fn view<'a>(&'a self, model: &'a Model) -> Element<'a, Message> {
        row![
            // Self::view_sidebar(self.series.borrow_mut(), model),
            cartesian(Heatmap::new(Box::new((self, model))), &self.plot_state).map(Message::Plot),
        ]
        .into()
    }

    fn update(&mut self, _model: &mut Model, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Plot(_) => {
                // self.plot_state.borrow_mut().update(msg);
                Command::none()
            }
        }
    }
}
