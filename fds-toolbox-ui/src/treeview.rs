use iced::{Element, Column, Scrollable, scrollable};

pub struct State<Key: Eq> {
    children: im::HashMap<Key, ElemState>,
    scroll: scrollable::State,
}

struct ElemState {
    visible: bool,
    selected: bool,
}

enum Message<Key> {
    Toggle(Key),
    Select(Key),
    SelectShift(Key),
    SelectAlt(Key),
    Scroll(f32),
}

impl<Key: Eq> State<Key> {
    pub fn new() -> Self {
        Self {
            children: im::HashMap::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn update(&mut self, msg: Message<Key>) {
        match msg {
            Message::Toggle(key) => {
                if let Some(state) = self.children.get_mut(&key) {
                    state.visible = !state.visible;
                }
            }
            Message::Select(key) => {
                if let Some(state) = self.children.get_mut(&key) {
                    state.selected = true;
                }
            }
            Message::SelectShift(key) => {
                if let Some(state) = self.children.get_mut(&key) {
                    state.selected = true;
                }
            }
            Message::SelectAlt(key) => {
                if let Some(state) = self.children.get_mut(&key) {
                    state.selected = true;
                }
            }
            Message::Scroll(scroll) => self.scroll.,
        }
    }

    // TODO: Really not a big fan of this signature
    pub fn view<'a, View: Fn(bool) -> Element<'a, Message<Key>>>(
        &mut self,
        tree: impl Iterator<Item = (Key, View)> + 'a,
    ) -> Element<'a, Message<Key>> {
        let mut col = Scrollable::new(&mut self.scroll)
            .on_scroll(Message::Scroll);

        for (key, view) in tree {
            if let Some(state) = self.children.get(&key) {
                if state.visible {
                    col = col.push(view());
                }
            }
        }

        col.into()
    }
}

enum Message {
    ToggleVisibility(usize),
    ToggleSelection(usize),
    ToggleCollapse(usize),
}