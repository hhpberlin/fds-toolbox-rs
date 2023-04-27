use std::{
    cell::RefCell,
    iter::{once, Once},
};

use fds_toolbox_lazy_data::{fs::AnyFs, sims::{Simulations, BySimulation}, sim::SliceIdx};
use iced::{widget::row, Command, Element};

use crate::{
    plotters::{
        cartesian::{self, cartesian},
        heatmap::Heatmap,
        ids::IdSource,
    },
    tabs::Tab,
};

#[derive(Debug)]
pub struct SliceTab {
    slice: BySimulation<SliceIdx>,
    frame: usize,
    plot_state: RefCell<cartesian::State>,
}

impl IdSource for SliceTab {
    type Id = BySimulation<SliceIdx>;
    type Iter<'a> = Once<Self::Id>
    where
        Self: 'a;

    fn iter_ids(&self) -> Self::Iter<'_> {
        once(self.slice)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Plot(cartesian::Message),
    // AddSlice(SliceFile),
}

impl SliceTab {
    pub fn new(slice: BySimulation<SliceIdx>) -> Self {
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

impl Tab<Simulations> for SliceTab {
    type Message = Message;

    fn title(&self) -> String {
        "Slice Plot".to_string()
    }

    fn view<'a>(&'a self, model: &'a Simulations) -> Element<'a, Message> {
        row![
            // Self::view_sidebar(self.series.borrow_mut(), model),
            cartesian(Heatmap::new(model, self, self.frame), &self.plot_state).map(Message::Plot),
        ]
        .into()
    }

    fn update(
        &mut self,
        _model: &mut Simulations,
        message: Self::Message,
    ) -> Command<Self::Message> {
        match message {
            Message::Plot(_) => {
                // self.plot_state.borrow_mut().update(msg);
                Command::none()
            }
        }
    }
}
