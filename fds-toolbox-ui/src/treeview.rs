use iced::{Column, Command, Element, Row, Text};

// Based on elm version: https://github.com/gribouille/elm-treeview/blob/master/src/Treeview.elm

#[derive(Debug)]
pub struct Tree<'a, Key: Eq> {
    id: Key,
    name: &'a str,
    collapsed: bool,
    selected: bool,
    children: Vec<Tree<'a, Key>>,
}

#[derive(Debug)]
pub struct TreeView<'a, Key: Eq> {
    tree: Tree<'a, Key>,
    info: TreeViewInfo<Key>,
}

#[derive(Debug)]
pub struct TreeViewInfo<Key: Eq> {
    /// The first selected element in the tree,
    /// used for shift selection.
    pub selection_root: Option<Key>,
    pub config: Config,
    pub search: String,
}

#[derive(Debug, Clone)]
enum TreeMessage<Key: Eq> {
    ToggleCollapsed(Key),
    Select(Key),
    SelectShift(Key),
    SelectAlt(Key),
    Search(String),
    DoubleClick(Key),
}

#[derive(Debug, Clone, Copy)]
struct Config {
    selection: ConfigSelection,
    search: ConfigSearch,
    sort: Sort,
}

#[derive(Debug, Clone, Copy)]
struct ConfigSelection {
    enabled: bool,
    multiple: bool,
    cascade: bool,
    cascade_shift: bool,
}

#[derive(Debug, Clone, Copy)]
struct ConfigSearch {
    enabled: bool,
}

#[derive(Debug, Clone, Copy)]
enum Sort {
    None,
    Ascending,
    Descending,
}

impl<'t, Key: Eq + Copy> TreeView<'t, Key> {
    fn update<'a: 't>(&'a mut self, message: TreeMessage<Key>) -> Command<TreeMessage<Key>> {
        match message {
            TreeMessage::ToggleCollapsed(ref id) => {
                if let Some(tree) = self.tree.get_mut(id) {
                    tree.collapsed = !tree.collapsed;
                }
            }
            TreeMessage::Select(ref id) => self.select(id),
            TreeMessage::SelectShift(ref id) => self.select_shift(id),
            TreeMessage::SelectAlt(ref id) => self.select_alt(id),
            TreeMessage::Search(str) => self.info.search = str,
            TreeMessage::DoubleClick(ref _id) => {}
        }
        Command::none()
    }

    fn select<'a: 't>(&'a mut self, id: &Key) {
        // Deselect everything
        self.tree.set_selected_rec(false);

        if let Some(tree) = self.tree.get_mut(id) {
            self.info.selection_root = Some(*id);
            tree.set_selected(true, self.info.config.selection.cascade);
        }
    }

    fn select_shift<'a: 't>(&'a mut self, id: &Key) {
        match self.info.selection_root {
            Some(ref root) => {
                // Deselect everything
                self.tree.set_selected_rec(false);

                self.tree
                    .region_op(self.info.config.sort, id, root, |tree| {
                        // TODO: Cascading is very inefficient like this
                        tree.set_selected(true, self.info.config.selection.cascade_shift);
                    });
            }
            None => self.select(id),
        }
    }

    fn select_alt<'a: 't>(&'a mut self, id: &Key) {
        if let Some(tree) = self.tree.get_mut(id) {
            tree.set_selected(true, self.info.config.selection.cascade);
        }
    }

    fn view(&self) -> Element<TreeMessage<Key>> {
        self.tree.view(&self.info)
    }
}

impl<'t, Key: Eq> Tree<'t, Key> {
    /// Recursively sets all children to be de/selected.
    fn set_selected_rec(&mut self, selected: bool) {
        self.selected = selected;
        for child in &mut self.children {
            child.set_selected_rec(selected);
        }
    }

    fn set_selected(&mut self, selected: bool, cascade: bool) {
        match cascade {
            true => self.set_selected_rec(selected),
            false => self.selected = selected,
        }
    }

    fn get(&self, id: &Key) -> Option<&Tree<Key>> {
        if self.id == *id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(result) = child.get(id) {
                return Some(result);
            }
        }
        None
    }

    fn get_mut<'a: 't>(&'a mut self, id: &Key) -> Option<&'a mut Tree<Key>> {
        if self.id == *id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(result) = child.get_mut(id) {
                return Some(result);
            }
        }
        None
    }

    fn view(&self, info: &TreeViewInfo<Key>) -> Element<TreeMessage<Key>> {
        let mut column = Column::new().push({
            let mut row = Row::new();
            if self.children.is_empty() {
                row = row.push(Text::new(if self.collapsed { ">" } else { "V" }))
                // TODO: Fancy icon
            }
            row = row.push(Text::new(self.name));
            row
        });
        if !self.collapsed {
            for child in &self.children {
                column = column.push(child.view(info));
            }
        }
        column.into()
    }

    fn iter_sorted(&self, sort: Sort) -> impl Iterator<Item = &Tree<Key>> {
        let mut children = self.children.iter().collect::<Vec<_>>();
        children.sort_by(|a, b| match sort {
            Sort::None => a.name.cmp(b.name),
            Sort::Ascending => a.name.cmp(b.name),
            Sort::Descending => b.name.cmp(a.name),
        });
        let mut result = Vec::new();
        for child in children {
            result.extend(child.iter_sorted(sort));
            result.push(child);
        }
        result.into_iter()
    }

    fn region_op<'a: 't>(
        &'a mut self,
        sort: Sort,
        id_start: &Key,
        id_end: &Key,
        op: impl Fn(&mut Tree<Key>),
    ) {
        self.region_op_internal(sort, id_start, id_end, op, Bnd::None);
    }

    fn region_op_internal<'a: 't>(
        &'a mut self,
        sort: Sort,
        id_start: &Key,
        id_end: &Key,
        op: impl Fn(&mut Tree<Key>),
        state: Bnd,
    ) -> Bnd {
        let mut result = state;
        if self.id == *id_start || self.id == *id_end {
            op(self);
            if *id_start == *id_end {
                return Bnd::End;
            } else {
                result = match result {
                    Bnd::None => Bnd::Inside,
                    Bnd::Inside => return Bnd::End,
                    Bnd::End => return Bnd::End, // unreachable
                };
            }
        }

        let mut children = self.children.iter_mut().collect::<Vec<_>>();
        children.sort_by(|a, b| match sort {
            Sort::None => a.name.cmp(b.name),
            Sort::Ascending => a.name.cmp(b.name),
            Sort::Descending => b.name.cmp(a.name),
        });

        for child in children {
            result = child.region_op_internal(sort, id_start, id_end, &op, result);
            if let Bnd::End = result {
                return Bnd::End;
            }
        }

        result
    }
}

enum Bnd {
    None,
    Inside,
    End,
}
