use iced::{
    widget::{button, row, text},
    Element,
};
use iced_aw::Icon;

trait TreeNode {
    type Message: Clone + 'static;
    type Children: TreeNode<Message = Self::Message>;
    // TODO: Track https://github.com/rust-lang/rust/issues/91611 to replace this with impl Trait
    type Iter: Iterator<Item = Self::Children>;

    fn children(&self) -> Self::Iter;

    // TODO: Find a better name for this
    // Does NOT need to match whether children() returns an empty iterator. Despite the name, this is more of a "can be expanded" flag.
    // This may be false on nodes with no children, as to represent a node that can typically be expanded but has no children in this specific case.
    // For example a directory with no files in it would return false here, despite technically being a leaf, in order to represent that it could theoretically have files in it.
    fn is_leaf(&self) -> bool {
        self.children().next().is_some()
    }

    // fn on_click(&self) -> impl Fn(bool) -> Self::Message;
    // (message when clicked while expanded, ... while collapsed)
    // I'd use the above if impl trait in trait return types was stable, but an associated type would be overkill
    fn on_click(&self) -> (Self::Message, Self::Message);

    // Selection state for leaves, expansion state for branches
    // None if not selectable/expandable
    fn is_selected(&self) -> Option<bool>;

    fn name(&self) -> String;
}

fn view_tree<'a, T: TreeNode + 'a>(root: T) -> Element<'a, T::Message> {
    fn view_node<'a, T: TreeNode + 'a>(node: &'a T) -> Element<'static, T::Message> {
        let (expanded, collapsed) = node.on_click();
        let is_selected = node.is_selected();
        let is_leaf = node.is_leaf();

        let icon = match (is_selected, is_leaf) {
            (Some(true), true) => Icon::CheckSquareFill,
            (Some(false), true) => Icon::CheckSquare,
            (None, true) => Icon::DashSquare,
            (Some(true), false) => Icon::ChevronDown,
            (Some(false), false) => Icon::ChevronRight,
            (None, false) => Icon::Dash,
        };
        let icon = text(icon).font(iced_aw::ICON_FONT);

        let button = button(row![icon, text(node.name())]);

        let button = match is_selected {
            Some(is_selected) => button.on_press(if is_selected { expanded } else { collapsed }),
            _ => button,
        };

        button.into()
    }

    view_node(&root)
}
