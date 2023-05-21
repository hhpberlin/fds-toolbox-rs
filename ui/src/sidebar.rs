use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
};

use fds_toolbox_core::{
    file::SliceSeriesIdx,
    formats::csv::devc::{DeviceIdx, DeviceList, DeviceReadings},
};
use fds_toolbox_lazy_data::moka::{
    MokaStore, P3dIdx, S3dIdx, SimulationDataIdx, SimulationIdx, SimulationsDataIdx,
};
use iced::{
    widget::{button, checkbox, text, Row, Space},
    Element, Length,
};
use iced_aw::Grid;
use itertools::Itertools;

use crate::app::Message;

// struct Selections {
//     expanded: HashSet<Group>,
//     selected0: HashSet<Series0>,
//     selected2: HashSet<Series2>,
//     selected3: HashSet<Series3>,
// }

// fn view(
//     store: MokaStore,
//     simulations: impl Iterator<Item = SimulationIdx>,
//     selected0: impl Fn(Series0) -> bool,
//     selected2: impl Fn(Series2) -> bool,
//     selected3: impl Fn(Series3) -> bool,
// ) -> Element<'_, Message> {
//     for sim_idx in simulations {
//         let sim = store.sim().try_get(sim_idx, ());
//         let name = match sim {
//             Some(sim) => sim.name.clone(),
//             // None => format!("Simulation {}", sim_idx),
//             None => match store.get_path_by_idx(sim_idx) {
//                 Some(path) => format!("{} (Unloaded)", path.chid),
//                 None => format!("Simulation {} (Unloaded, Error)", sim_idx),
//             },
//         };
//         checkbox(name, is_checked, f)
//     }
// }

#[derive(Debug)]
struct SelectionState<Message> {
    selected: bool,
    msg: Option<SelectionMessages<Message>>,
}

impl<Message> SelectionState<Message> {
    pub fn new(selected: bool) -> Self {
        Self {
            selected,
            msg: None,
        }
    }

    pub fn with_msg(self, msg: SelectionMessages<Message>) -> Self {
        Self {
            msg: Some(msg),
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SelectionMessages<Message> {
    on: Message,
    off: Message,
}

impl<Message> SelectionMessages<Message> {
    pub fn new(on: Message, off: Message) -> Self {
        Self { on, off }
    }

    pub fn from_fn(msg: impl Fn(bool) -> Message) -> Self {
        Self {
            on: msg(true),
            off: msg(false),
        }
    }

    pub fn get(&self, selected: bool) -> &Message {
        if selected {
            &self.on
        } else {
            &self.off
        }
    }
}

pub enum Thing<'a, Message> {
    Group {
        name: String,
        expanded: SelectionState<Message>,
        loaded: SelectionState<Message>,
        data: Option<DynIter<'a, Thing<'a, Message>>>,
    },
    Element {
        name: String,
        selected: SelectionState<Message>,
        loaded: SelectionState<Message>,
    },
}

type DynIter<'a, T> = Box<dyn Iterator<Item = T> + 'a>;

fn box_iter<'a, T>(iter: impl IntoIterator<Item = T> + 'a) -> DynIter<'a, T> {
    Box::new(iter.into_iter()) as _
}

// enum LazyDynIter<'a, T> {
//     Eager(DynIter<'a, T>),
//     Lazy(Box<dyn FnOnce() -> DynIter<'a, T> + 'a>),
// }

// impl<'a, T> IntoIterator for LazyDynIter<'a, T> {
//     type Item = T;
//     type IntoIter = DynIter<'a, T>;

//     fn into_iter(self) -> Self::IntoIter {
//         match self {
//             Self::Eager(iter) => iter,
//             Self::Lazy(f) => f(),
//         }
//     }
// }

// impl<'a, T> LazyDynIter<'a, T> {
//     pub fn eager(iter: impl IntoIterator<Item = T> + 'a) -> Self {
//         Self::Eager(Box::new(iter.into_iter()))
//     }

//     pub fn lazy<Iter: IntoIterator<Item = T> + 'a>(f: impl FnOnce() -> Iter + 'a) -> Self {
//         Self::Lazy(Box::new(move || Box::new(f().into_iter())))
//     }
// }

// impl<Iter: IntoIterator> From<Iter> for LazyDynIter<'_, Iter::Item> {
//     fn from(iter: Iter) -> Self {
//         LazyDynIter::Eager(Box::new(iter.into_iter()) as _)
//     }
// }

// impl<Fn: FnOnce() -> Iter, Iter: Iterator> From<Fn> for LazyDynIter<'_, Iter::Item> {
//     fn from(f: Fn) -> Self {
//         LazyDynIter::Lazy(Box::new(move || Box::new(f()) as _) as _)
//     }
// }

impl<Message: Debug> Debug for Thing<'_, Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Group {
                name,
                expanded,
                loaded,
                data,
            } => f
                .debug_struct("Group")
                .field("name", name)
                .field("expanded", expanded)
                .field("loaded", loaded)
                // .field("data", data)
                .finish(),
            Self::Element {
                name,
                selected,
                loaded,
            } => f
                .debug_struct("Element")
                .field("name", name)
                .field("selected", selected)
                .field("loaded", loaded)
                .finish(),
        }
    }
}

pub trait SelectionSrc<Message> {
    fn simulation(&self, idx: SimulationIdx) -> SelectionState<Message>;
    fn group(&self, idx: SimulationGroup) -> SelectionState<Message>;
    fn group_quantity(&self, idx: SimulationGroupQuantity) -> SelectionState<Message>;
    fn series0(&self, idx: Series0) -> SelectionState<Message>;
    fn series2(&self, idx: Series2) -> SelectionState<Message>;
    fn series3(&self, idx: Series3) -> SelectionState<Message>;

    fn load_msg(&self, idx: SimulationsDataIdx) -> SelectionMessages<Message>;
}

#[derive(Clone, Copy)]
pub struct Dummy;
impl SelectionSrc<()> for Dummy {
    fn simulation(&self, idx: SimulationIdx) -> SelectionState<()> {
        SelectionState {
            selected: true,
            msg: None,
        }
    }

    fn group(&self, idx: SimulationGroup) -> SelectionState<()> {
        SelectionState {
            selected: true,
            msg: None,
        }
    }

    fn group_quantity(&self, idx: SimulationGroupQuantity) -> SelectionState<()> {
        SelectionState {
            selected: true,
            msg: None,
        }
    }

    fn series0(&self, idx: Series0) -> SelectionState<()> {
        SelectionState {
            selected: true,
            msg: None,
        }
    }

    fn series2(&self, idx: Series2) -> SelectionState<()> {
        SelectionState {
            selected: true,
            msg: None,
        }
    }

    fn series3(&self, idx: Series3) -> SelectionState<()> {
        SelectionState {
            selected: true,
            msg: None,
        }
    }

    fn load_msg(&self, idx: SimulationsDataIdx) -> SelectionMessages<()> {
        SelectionMessages { on: (), off: () }
    }
}

fn load_msg(idx: SimulationsDataIdx) -> SelectionMessages<Message> {
    SelectionMessages::new(Message::Load(idx), Message::Unload(idx))
}

pub fn simulation<'a, Message: 'static>(
    store: &'a MokaStore,
    sel_src: impl SelectionSrc<Message> + Copy + 'a,
    idx: SimulationIdx,
) -> Thing<'a, Message> {
    let sim = store.sim().try_get(idx, ());
    let name = match &sim {
        Some(sim) => sim.path.chid.clone(),
        // None => format!("Simulation {}", sim_idx),
        None => match store.get_path_by_idx(idx) {
            Some(path) => format!("{} (Unloaded)", path.chid),
            None => format!("Simulation {:?} (Unloaded, Error)", idx),
        },
    };
    let expanded = sel_src.simulation(idx);
    Thing::Group {
        name,
        expanded,
        loaded: SelectionState::new(sim.is_some())
            .with_msg(sel_src.load_msg(SimulationsDataIdx(idx, SimulationDataIdx::Simulation))),
        data: sim.map(|_| box_iter([devices(store, sel_src, idx)]) as _),
    }
}

// struct SelfBorrowingIter<Owned, IterFn: FnOnce(&Owned) -> Iter, Iter: Iterator> {
//     owned: Owned,
//     iter: Option<Iter>,
//     iter_fn: IterFn,
// }
// impl<Owned, IterFn: FnOnce(&Owned) -> Iter, Iter: Iterator> SelfBorrowingIter<Owned, IterFn, Iter> {
//     pub fn new(owned: Owned, iter_fn: IterFn) -> Self {
//         let mut this = Self {
//             owned,
//             iter: None,
//             iter_fn,
//         };
//         this.iter = Some((this.iter_fn)(&this.owned));
//         this
//     }
// }

// impl<Owner, Iter: Iterator> Iterator for SelfBorrowingIter<Owner, Iter> {
//     type Item = Iter::Item;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.1.next()
//     }
// }

fn devices<'a, Message: 'static>(
    store: &'a MokaStore,
    sel_src: impl SelectionSrc<Message> + Copy + 'a,
    idx: SimulationIdx,
) -> Thing<'a, Message> {
    let devc = store.devc().try_get(idx, ());

    let data = devc.map(|devc| {
        // TODO: This assumes all elements to be sorted by group,
        //        group_by only groups consecutive elements with the same key.
        //       Normalizing the order and guaranteeing it is sorted inside of `DeviceList` would be best probably.

        let by_qty_collected = {
            let by_qty = devc
                .enumerate_device_readings()
                .into_group_map_by(|x| Quantity::from_str(&x.1.unit));

            let iter = by_qty
                .into_iter()
                // .map(|(qty, grp)| (qty, grp.into_iter().map(|(idx, devc)| devices_qty(store, sel_src, idx, qty, idxs)).collect::<Vec<_>>()))
                .map(|(qty, grp)| devices_qty(store, sel_src, idx, qty, grp));

            iter.collect::<Vec<_>>()
        };

        box_iter(
            by_qty_collected, // .into_iter()
                              // .map(move |(qty, elems)| devices_qty(store, sel_src, idx, qty, elems)),
        )
    });

    Thing::Group {
        name: "Devices".to_string(),
        expanded: sel_src.group(SimulationGroup(idx, Group::Series0)),
        loaded: SelectionState::new(data.is_some())
            .with_msg(sel_src.load_msg(SimulationsDataIdx(idx, SimulationDataIdx::DevciceList))),
        data,
    }
}

fn devices_qty<'a, 'd, Message: 'static>(
    store: &'a MokaStore,
    sel_src: impl SelectionSrc<Message> + Copy + 'a,
    idx: SimulationIdx,
    qty: Option<Quantity>,
    idxs: impl IntoIterator<Item = (DeviceIdx, &'d DeviceReadings)> + 'd,
) -> Thing<'a, Message> {
    Thing::Group {
        name: qty.map(Quantity::to_str).unwrap_or("Other").to_string(),
        expanded: sel_src.group_quantity(SimulationGroupQuantity(
            SimulationGroup(idx, Group::Series0),
            qty,
        )),
        loaded: SelectionState::new(true)
            .with_msg(sel_src.load_msg(SimulationsDataIdx(idx, SimulationDataIdx::DevciceList))),
        data: Some(box_iter(
            idxs.into_iter()
                .map(move |(devc_idx, devc)| device(store, sel_src, devc, (idx, devc_idx)))
                // TODO: Avoid collecting here
                .collect::<Vec<_>>(),
        )),
    }
}

fn device<'a, 'd, Message: 'static>(
    store: &'a MokaStore,
    sel_src: impl SelectionSrc<Message>,
    device: &'d DeviceReadings,
    idx: (SimulationIdx, DeviceIdx),
) -> Thing<'a, Message> {
    Thing::Element {
        name: device.name.clone(),
        selected: sel_src.series0(Series0::Device(idx.0, idx.1)),
        loaded: SelectionState::new(true)
            .with_msg(sel_src.load_msg(SimulationsDataIdx(idx.0, SimulationDataIdx::DevciceList))),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimulationGroup(SimulationIdx, Group);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Group {
    // Overview,
    Series0,
    Series2,
    Series3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quantity {
    Temperature,
    VolumeFlowRate,
    ReciprocalLength,
    // Other(&'static str),
}

impl Quantity {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "C" => Some(Quantity::Temperature),
            "m3/s" => Some(Quantity::VolumeFlowRate),
            "1/m" => Some(Quantity::ReciprocalLength),
            _ => None,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Quantity::Temperature => "C",
            Quantity::VolumeFlowRate => "m3/s",
            Quantity::ReciprocalLength => "1/m",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Series0Type {
    Device,
    Hrr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Series0 {
    Device(SimulationIdx, DeviceIdx),
    Hrr(SimulationIdx, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Series2Type {
    Slice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Series2 {
    Slice {
        sim: SimulationIdx,
        idx: SliceSeriesIdx,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Series3Type {
    S3D,
    P3D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Series3 {
    S3D(SimulationIdx, S3dIdx),
    P3D(SimulationIdx, P3dIdx),
}

impl<'a, Msg: Clone + 'a> Thing<'a, Msg> {
    fn view_self(&self) -> Element<'a, Msg> {
        match self {
            Thing::Group {
                name,
                expanded,
                loaded,
                ..
            } => {
                if loaded.selected {
                    // Draw a checkbox to select/deselect the group
                    match &expanded.msg {
                        Some(msg) => {
                            let msg = msg.clone();
                            checkbox(name, expanded.selected, move |x| msg.get(x).clone()).into()
                        }
                        None => text(name).into(),
                    }
                } else {
                    // Draw a loading button
                    match &loaded.msg {
                        Some(msg) => button(text(name)).on_press(msg.on.clone()).into(),
                        None => text(name).into(),
                    }
                }
            }
            Thing::Element {
                name,
                selected,
                loaded,
            } => {
                if loaded.selected {
                    // Draw a checkbox to select/deselect the element
                    match &selected.msg {
                        Some(msg) => {
                            let msg = msg.clone();
                            checkbox(name, selected.selected, move |x| msg.get(x).clone()).into()
                        }
                        None => text(name).into(),
                    }
                } else {
                    // Draw a loading button
                    match &loaded.msg {
                        Some(msg) => button(text(name)).on_press(msg.on.clone()).into(),
                        None => text(name).into(),
                    }
                }
            }
        }
    }

    // TODO: Apply theme
    fn view_all(self, offset: usize, insert: &mut impl FnMut(Element<'a, Msg>)) {
        insert(Space::new(Length::Fixed(offset as f32 * 10.), Length::Shrink).into());
        insert(self.view_self().into());
        if let Thing::Group {
            data: Some(data), ..
        } = self
        {
            for child in data {
                child.view_all(offset + 1, insert);
            }
        }
    }

    pub fn view(self) -> Element<'a, Msg>
    where
        // TODO: I have no clue why it complains if Msg is just 'a and not 'static
        Msg: 'static,
    {
        let mut grid = Grid::with_columns(2);

        self.view_all(0, &mut |e| grid.insert(e));
        grid.into()
    }
}
