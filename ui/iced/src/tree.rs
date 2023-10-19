use fds_toolbox_lazy_data::moka::{MokaStore, SimulationIdx};
use iced::{
    widget::{button, row, text, Row, Text},
    Element,
};
use iced_aw::Icon;

use crate::app::Message;

enum Selection {
    Selectable(bool),
    Unselectable,
}

fn get_icon(is_selected: Selection, is_leaf_type: bool) -> Icon {
    match (is_selected, is_leaf_type) {
        (Selection::Selectable(true), true) => Icon::CheckSquareFill,
        (Selection::Selectable(false), true) => Icon::CheckSquare,
        (Selection::Unselectable, true) => Icon::DashSquare,
        (Selection::Selectable(true), false) => Icon::ChevronDown,
        (Selection::Selectable(false), false) => Icon::ChevronRight,
        (Selection::Unselectable, false) => Icon::Dash,
    }
}

fn icon(is_selected: Selection, is_leaf: bool) -> Text<'static> {
    text(get_icon(is_selected, is_leaf)).font(iced_aw::ICON_FONT)
}

struct TreeWriter<'a> {
    row: Row<'static, Message>,
    model: &'a MokaStore,
    current_level: usize,
}

impl TreeWriter<'_> {
    fn indent(&mut self) {
        self.current_level += 1;
    }

    fn dedent(&mut self) {
        self.current_level -= 1;
    }

    pub fn new(model: &'_ MokaStore) -> Self {
        Self {
            row: row!(),
            model,
            current_level: 0,
        }
    }

    fn node(is_selected: Selection, is_leaf_type: bool) {
        self.row = self.row.push(child);
    }

    fn full(&mut self) {
    }

    pub fn into_inner(self) -> Row<'static, Message> {
        self.row
    }
}

fn tree(row: &mut Row<Message>, model: &MokaStore) {
    row
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
