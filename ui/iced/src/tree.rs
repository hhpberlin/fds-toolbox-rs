use std::collections::HashMap;

use fds_toolbox_lazy_data::moka::{MokaStore, SimulationIdx};
use iced::{
    widget::{button, checkbox, column, pick_list, row, text, Column, Row, Space, Text},
    Element, Length,
};
use iced_aw::Icon;

use crate::{app::Message, plotters::ids::SeriesSourceLine};

// enum Selection {
//     Selectable(bool),
//     Unselectable,
// }

// fn get_icon(is_selected: Selection, is_leaf_type: bool) -> Icon {
//     match (is_selected, is_leaf_type) {
//         (Selection::Selectable(true), true) => Icon::CheckSquareFill,
//         (Selection::Selectable(false), true) => Icon::CheckSquare,
//         (Selection::Unselectable, true) => Icon::DashSquare,
//         (Selection::Selectable(true), false) => Icon::ChevronDown,
//         (Selection::Selectable(false), false) => Icon::ChevronRight,
//         (Selection::Unselectable, false) => Icon::Dash,
//     }
// }

fn icon_text(icon: Icon) -> Text<'static> {
    text(icon).font(iced_aw::ICON_FONT)
}

pub struct TreeWriter<'a, Message> {
    row: Option<Column<'a, Message>>,
    current_level: usize,
}

impl<'a> TreeWriter<'a, Message> {
    fn indent(&mut self) {
        self.current_level += 1;
    }

    fn dedent(&mut self) {
        self.current_level -= 1;
    }

    pub fn new() -> Self {
        Self {
            row: Some(column!()),
            current_level: 0,
        }
    }

    fn add_node(
        &mut self,
        icon: Icon,
        content: impl Into<Element<'a, Message>>,
        msg: Option<Message>,
    ) {
        let node = row![icon_text(icon), content.into()];
        // let hi = checkbox(label, is_checked, f);
        // let hi = pick_list(options, selected, on_selected);

        self.row = Some(self.row.take().unwrap().push(row![
            Space::new(Length::Fixed(self.current_level as f32 * 10.), 0.0),
            button(node).on_press_maybe(msg)
        ]));
    }

    fn add_node_and_children(
        &mut self,
        icon: Icon,
        content: impl Into<Element<'a, Message>>,
        msg: Option<Message>,
        children: impl FnOnce(&mut Self),
    ) {
        self.add_node(icon, content, msg);
        self.indent();
        children(self);
        self.dedent();
    }

    pub fn into_inner(self) -> Column<'a, Message> {
        self.row.unwrap()
    }
}

fn chevron(sel: bool) -> Icon {
    if sel {
        Icon::ChevronDown
    } else {
        Icon::ChevronRight
    }
}

fn check(sel: bool) -> Icon {
    if sel {
        Icon::CheckSquareFill
    } else {
        Icon::CheckSquare
    }
}

#[derive(Debug, Default)]
pub struct SimsSelection {
    // This is never cleaned up, so it could *technically* leak memory, but never by a significant amount
    by_sim: HashMap<SimulationIdx, (bool, SimSelection)>,
}

#[derive(Debug, Clone)]
pub enum SimsSelectionMessage {
    Select(SimulationIdx, bool),
    Inner(SimulationIdx, SimSelectionMessage),
}

impl SimsSelection {
    pub fn update(&mut self, msg: SimsSelectionMessage) {
        dbg!(&msg);
        match msg {
            SimsSelectionMessage::Select(i, x) => self.by_sim.entry(i).or_default().0 = x,
            SimsSelectionMessage::Inner(i, msg) => self.by_sim.entry(i).or_default().1.update(msg),
        }
    }

    // pub fn iter_selected_lines(&self, s: &MokaStore) -> impl Iterator {
    //     self.by_sim.iter().flat_map(|(sim, (_, sel))| {
    //         let devc = s.devc().try_get(*sim, ());
    //         sel.line_inner.devc_inner.selected.iter().enumerate().filter(|(_, sel)| **sel)
    //         // TODO: Inform user about unloaded plots somehow
    //         .filter_map(|(i, _)| devc.map(|x| x.devices[i].values))
    //     })
    // }
}

impl SeriesSourceLine for (&SimsSelection, &MokaStore) {
    fn for_each_series(
        &self,
        f: &mut dyn for<'view> FnMut(fds_toolbox_core::common::series::TimeSeries0View<'view>),
    ) {
        self.0.by_sim.iter().for_each(|(sim, (_, sel))| {
            let devc = self.1.devc().try_get(*sim, ());
            if let Some(devc) = devc {
                for (sel, device) in sel
                    .line_inner
                    .devc_inner
                    .selected
                    .iter()
                    .zip(devc.iter_device_views())
                {
                    if *sel {
                        f(device);
                    }
                }
            }
        });
    }
}

pub fn root(
    tree: &mut TreeWriter<'_, Message>,
    sel: &SimsSelection,
    model: &MokaStore,
    sims: &mut dyn Iterator<Item = SimulationIdx>,
    msg_map: impl Fn(SimsSelectionMessage) -> Message,
) {
    let empty_sel = SimSelection::default();
    let empty_sel = &(false, empty_sel);

    for sim_idx in sims {
        let (selected, sel) = sel.by_sim.get(&sim_idx).unwrap_or(empty_sel);

        tree.add_node_and_children(
            chevron(*selected),
            text(match model.sim().try_get(sim_idx, ()) {
                Some(x) => x.smv.chid.clone(),
                None => "Loading...".to_string(),
            }),
            Some(msg_map(SimsSelectionMessage::Select(sim_idx, !selected))),
            |tree| {
                if !selected {
                    return;
                }
                sim(tree, sel, model, sim_idx, |msg| {
                    msg_map(SimsSelectionMessage::Inner(sim_idx, msg))
                })
            },
        );
    }
}

#[derive(Debug, Default)]
pub struct SimSelection {
    info: bool,
    line: bool,
    line_inner: LineSelection,
    slice: bool,
    volume: bool,
}

#[derive(Debug, Clone)]
pub enum SimSelectionMessage {
    Info(bool),
    Line(bool),
    LineInner(LineSelectionMessage),
    Slice(bool),
    Volume(bool),
}

impl SimSelection {
    pub fn update(&mut self, msg: SimSelectionMessage) {
        match msg {
            SimSelectionMessage::Info(x) => self.info = x,
            SimSelectionMessage::Line(x) => self.line = x,
            SimSelectionMessage::LineInner(msg) => self.line_inner.update(msg),
            SimSelectionMessage::Slice(x) => self.slice = x,
            SimSelectionMessage::Volume(x) => self.volume = x,
        }
    }
}

pub fn sim(
    tree: &mut TreeWriter<'_, Message>,
    sel: &SimSelection,
    model: &MokaStore,
    sim_idx: SimulationIdx,
    msg_map: impl Fn(SimSelectionMessage) -> Message,
) {
    // let sim = model.sim().try_get(sim_idx, ());

    tree.add_node(
        check(sel.info),
        text("Info"),
        Some(msg_map(SimSelectionMessage::Info(!sel.info))),
    );
    tree.add_node_and_children(
        check(sel.line),
        text("Line"),
        Some(msg_map(SimSelectionMessage::Line(!sel.line))),
        |tree| {
            if !sel.line {
                return;
            }
            line(tree, &sel.line_inner, model, sim_idx, |msg| {
                msg_map(SimSelectionMessage::LineInner(msg))
            })
        },
    );
    tree.add_node(
        check(sel.slice),
        text("Slice"),
        Some(msg_map(SimSelectionMessage::Slice(!sel.slice))),
    );
    tree.add_node(
        check(sel.volume),
        text("Volume"),
        Some(msg_map(SimSelectionMessage::Volume(!sel.volume))),
    );
}

#[derive(Debug, Default)]
pub struct LineSelection {
    devc: bool,
    devc_inner: DevcSelection,
}

#[derive(Debug, Clone)]
pub enum LineSelectionMessage {
    Devc(bool),
    DevcInner(DevcSelectionMessage),
}

impl LineSelection {
    pub fn update(&mut self, msg: LineSelectionMessage) {
        match msg {
            LineSelectionMessage::Devc(x) => self.devc = x,
            LineSelectionMessage::DevcInner(msg) => self.devc_inner.update(msg),
        }
    }
}

pub fn line(
    tree: &mut TreeWriter<'_, Message>,
    sel: &LineSelection,
    model: &MokaStore,
    sim_idx: SimulationIdx,
    msg_map: impl Fn(LineSelectionMessage) -> Message,
) {
    tree.add_node_and_children(
        check(sel.devc),
        text("Devc"),
        Some(msg_map(LineSelectionMessage::Devc(!sel.devc))),
        |tree| {
            if !sel.devc {
                return;
            }
            devc(tree, &DevcSelection::default(), model, sim_idx, |msg| {
                msg_map(LineSelectionMessage::DevcInner(msg))
            })
        },
    );
}

#[derive(Debug, Default)]
pub struct DevcSelection {
    selected: Vec<bool>,
}

#[derive(Debug, Clone)]
pub enum DevcSelectionMessage {
    Select(usize, bool),
}

impl DevcSelection {
    pub fn update(&mut self, msg: DevcSelectionMessage) {
        match msg {
            DevcSelectionMessage::Select(i, x) => {
                if self.selected.len() <= i {
                    self.selected
                        .extend(std::iter::repeat(false).take(i - self.selected.len() + 1))
                }
                self.selected[i] = x;
            }
        }
    }
}

pub fn devc(
    tree: &mut TreeWriter<'_, Message>,
    sel: &DevcSelection,
    model: &MokaStore,
    sim_idx: SimulationIdx,
    msg_map: impl Fn(DevcSelectionMessage) -> Message,
) {
    let devc = model.devc().try_get(sim_idx, ());

    if let Some(devc) = devc {
        for (i, device) in devc.devices.iter().enumerate() {
            let sel = sel.selected.get(i).copied().unwrap_or(false);
            tree.add_node(
                check(sel),
                text(format!("{} ({})", device.name, device.unit)),
                Some(msg_map(DevcSelectionMessage::Select(i, !sel))),
            );
        }
    } else {
        tree.add_node(Icon::Dash, text("Loading..."), None);
    }
}

/*
pub trait TreeNode<Model> {
    type Message: Clone + 'static;
    type Children: TreeNode<Model, Message = Self::Message>;

    fn children(&self, model: &Model) -> impl Iterator<Item = Self::Children>;

    // TODO: Find a better name for this
    // Does NOT need to match whether children() returns an empty iterator. Despite the name, this is more of a "can be expanded" flag.
    // This may be false on nodes with no children, as to represent a node that can typically be expanded but has no children in this specific case.
    // For example a directory with no files in it would return false here, despite technically being a leaf, in order to represent that it could theoretically have files in it.
    fn is_leaf(&self, model: &Model) -> bool {
        self.children(model).next().is_some()
    }

    // fn on_click(&self) -> impl Fn(bool) -> Self::Message;
    // (message when clicked while expanded, ... while collapsed)
    // I'd use the above if impl trait in trait return types was stable, but an associated type would be overkill
    fn on_click(&self, model: &Model) -> (Self::Message, Self::Message);

    // Selection state for leaves, expansion state for branches
    // None if not selectable/expandable
    fn is_selected(&self, model: &Model) -> Option<bool>;

    fn name(&self, model: &Model) -> String;
}

pub fn view_tree<'a, M, T: TreeNode<M> + 'a>(root: T, model: &M) -> Element<'a, T::Message> {
    fn view_node<'a, M, T: TreeNode<M> + 'a>(node: &'a T, m: &M) -> Element<'static, T::Message> {
        let (expanded, collapsed) = node.on_click(m);
        let is_selected = node.is_selected(m);
        let is_leaf = node.is_leaf(m);

        let icon = match (is_selected, is_leaf) {
            (Some(true), true) => Icon::CheckSquareFill,
            (Some(false), true) => Icon::CheckSquare,
            (None, true) => Icon::DashSquare,
            (Some(true), false) => Icon::ChevronDown,
            (Some(false), false) => Icon::ChevronRight,
            (None, false) => Icon::Dash,
        };
        let icon = text(icon).font(iced_aw::ICON_FONT);

        let button = button(row![icon, text(node.name(m))]);

        let button = match is_selected {
            Some(is_selected) => button.on_press(if is_selected { expanded } else { collapsed }),
            _ => button,
        };

        button.into()
    }

    view_node(&root, model)
}

enum Dimensionality {
    Info,
    Line,
    Slice,
    Volume,
}

impl TreeNode<MokaStore> for SimulationIdx {
    type Message = Message;

    type Children = Dimensionality;

    fn children(&self, m: &MokaStore) -> impl Iterator<Item = Self::Children> {
        [Dimensionality::Info, Dimensionality::Line, Dimensionality::Slice, Dimensionality::Volume].into_iter()
    }

    fn on_click(&self, model: &MokaStore) -> (Self::Message, Self::Message) {
        todo!()
    }

    fn is_selected(&self, model: &MokaStore) -> Option<bool> {
        todo!()
    }

    fn name(&self, model: &MokaStore) -> String {
        todo!()
    }

}

// impl TreeNode<MokaStore> for Dimensionality {
//     type Message = Message;

//     type Children ;

//     type Iter;

//     fn children(&self, model: &MokaStore) -> Self::Iter {
//         todo!()
//     }

//     fn on_click(&self, model: &MokaStore) -> (Self::Message, Self::Message) {
//         todo!()
//     }

//     fn is_selected(&self, model: &MokaStore) -> Option<bool> {
//         todo!()
//     }

//     fn name(&self, model: &MokaStore) -> String {
//         todo!()
//     }
// }

*/
