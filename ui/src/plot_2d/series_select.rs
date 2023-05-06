use std::collections::HashSet;

use fds_toolbox_lazy_data::moka::SimulationIdx;
use iced::Element;

use crate::Model;

// pub struct LabeledSeries {
//     name: String,
//     // color: Box<dyn Into<ShapeStyle>>,
//     data: Box<dyn Iterator<Item = (f32, f32)>>,
// }

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

enum Series {
    Device { sim: SimulationIdx, idx: usize },
    // Slice { sim: SimulationIdx, idx: usize, x: usize, y: usize },
}

// impl Series {
//     pub fn view(&self, model: &Model) -> Option<TimeSeries0View> {
//         match *self {
//             Self::Device { sim, idx } => {
//                 match model.store.get_devc(sim).now_or_never() {
//                     Some(Ok(devc)) => devc.view_device_by_idx(idx),
//                     _ => None,
//                 }
//             }
//         }
//     }
// }

impl SeriesSelect {
    pub fn view<'a>(&self, _model: &'a Model) -> Element<'a, SeriesMessage> {
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

        thing.into()
    }
}
