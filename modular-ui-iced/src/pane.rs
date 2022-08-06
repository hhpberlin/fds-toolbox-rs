// From https://github.com/iced-rs/iced/tree/master/examples/pane_grid

use iced::alignment::{self, Alignment};
use iced::button::{self, Button};
use iced::executor;
use iced::keyboard;
use iced::pane_grid::{self, PaneGrid};
use iced::scrollable::{self, Scrollable};
use iced::{
    Application, Color, Column, Command, Container, Element, Length, Row, Settings, Size,
    Subscription, Text,
};
use iced_lazy::responsive::{self, Responsive};
use iced_native::{event, subscription, Event};

use crate::theme::THEME;
use crate::{theme, style};

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    panes: pane_grid::State<Pane>,
    panes_created: usize,
    focus: Option<pane_grid::Pane>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Split(pane_grid::Axis, pane_grid::Pane),
    SplitFocused(pane_grid::Axis),
    FocusAdjacent(pane_grid::Direction),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    TogglePin(pane_grid::Pane),
    Close(pane_grid::Pane),
    CloseFocused,
}

impl Application for Example {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let (panes, _) = pane_grid::State::new(Pane::new(0));

        (
            Example {
                panes,
                panes_created: 1,
                focus: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Pane grid - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Split(axis, pane) => {
                let result = self.panes.split(axis, &pane, Pane::new(self.panes_created));

                if let Some((pane, _)) = result {
                    self.focus = Some(pane);
                }

                self.panes_created += 1;
            }
            Message::SplitFocused(axis) => {
                if let Some(pane) = self.focus {
                    let result = self.panes.split(axis, &pane, Pane::new(self.panes_created));

                    if let Some((pane, _)) = result {
                        self.focus = Some(pane);
                    }

                    self.panes_created += 1;
                }
            }
            Message::FocusAdjacent(direction) => {
                if let Some(pane) = self.focus {
                    if let Some(adjacent) = self.panes.adjacent(&pane, direction) {
                        self.focus = Some(adjacent);
                    }
                }
            }
            Message::Clicked(pane) => {
                self.focus = Some(pane);
            }
            Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(&split, ratio);
            }
            Message::Dragged(pane_grid::DragEvent::Dropped { pane, target }) => {
                self.panes.swap(&pane, &target);
            }
            Message::Dragged(_) => {}
            Message::TogglePin(pane) => {
                if let Some(Pane { is_pinned, .. }) = self.panes.get_mut(&pane) {
                    *is_pinned = !*is_pinned;
                }
            }
            Message::Close(pane) => {
                if let Some((_, sibling)) = self.panes.close(&pane) {
                    self.focus = Some(sibling);
                }
            }
            Message::CloseFocused => {
                if let Some(pane) = self.focus {
                    if let Some(Pane { is_pinned, .. }) = self.panes.get(&pane) {
                        if !is_pinned {
                            if let Some((_, sibling)) = self.panes.close(&pane) {
                                self.focus = Some(sibling);
                            }
                        }
                    }
                }
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, status| {
            if let event::Status::Captured = status {
                return None;
            }

            match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    modifiers,
                    key_code,
                }) if modifiers.command() => handle_hotkey(key_code),
                _ => None,
            }
        })
    }

    fn view(&mut self) -> Element<Message> {
        let focus = self.focus;
        let total_panes = self.panes.len();

        let pane_grid = PaneGrid::new(&mut self.panes, |id, pane| {
            let is_focused = focus == Some(id);

            let Pane {
                responsive,
                pin_button,
                is_pinned,
                content,
                ..
            } = pane;

            let text = if *is_pinned { "Unpin" } else { "Pin" };
            let pin_button = Button::new(pin_button, Text::new(text).size(14))
                .on_press(Message::TogglePin(id))
                .style(style::Button::Pin)
                .padding(3);

            let title = Row::with_children(vec![
                pin_button.into(),
                Text::new("Pane").into(),
                Text::new(content.id.to_string())
                    .color(if is_focused {
                        THEME.pane_id_color_focused
                    } else {
                        THEME.pane_id_color_unfocused
                    })
                    .into(),
            ])
            .spacing(5);

            let title_bar = pane_grid::TitleBar::new(title)
                .controls(pane.controls.view(id, total_panes, *is_pinned))
                .padding(10)
                .style(if is_focused {
                    style::TitleBar::Focused
                } else {
                    style::TitleBar::Active
                });

            pane_grid::Content::new(Responsive::new(responsive, move |size| {
                content.view(id, total_panes, *is_pinned, size)
            }))
            .title_bar(title_bar)
            .style(if is_focused {
                style::Pane::Focused
            } else {
                style::Pane::Active
            })
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10)
        .on_click(Message::Clicked)
        .on_drag(Message::Dragged)
        .on_resize(10, Message::Resized);

        Container::new(pane_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }
}

fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<Message> {
    use keyboard::KeyCode;
    use pane_grid::{Axis, Direction};

    let direction = match key_code {
        KeyCode::Up => Some(Direction::Up),
        KeyCode::Down => Some(Direction::Down),
        KeyCode::Left => Some(Direction::Left),
        KeyCode::Right => Some(Direction::Right),
        _ => None,
    };

    match key_code {
        KeyCode::V => Some(Message::SplitFocused(Axis::Vertical)),
        KeyCode::H => Some(Message::SplitFocused(Axis::Horizontal)),
        KeyCode::W => Some(Message::CloseFocused),
        _ => direction.map(Message::FocusAdjacent),
    }
}

struct Pane {
    pub responsive: responsive::State,
    pub is_pinned: bool,
    pub pin_button: button::State,
    pub content: Content,
    pub controls: Controls,
}

struct Content {
    id: usize,
    scroll: scrollable::State,
    split_horizontally: button::State,
    split_vertically: button::State,
    close: button::State,
}

struct Controls {
    close: button::State,
}

impl Pane {
    fn new(id: usize) -> Self {
        Self {
            responsive: responsive::State::new(),
            is_pinned: false,
            pin_button: button::State::new(),
            content: Content::new(id),
            controls: Controls::new(),
        }
    }
}

impl Content {
    fn new(id: usize) -> Self {
        Content {
            id,
            scroll: scrollable::State::new(),
            split_horizontally: button::State::new(),
            split_vertically: button::State::new(),
            close: button::State::new(),
        }
    }
    fn view(
        &mut self,
        pane: pane_grid::Pane,
        total_panes: usize,
        is_pinned: bool,
        size: Size,
    ) -> Element<Message> {
        let Content {
            scroll,
            split_horizontally,
            split_vertically,
            close,
            ..
        } = self;

        let button = |state, label, message, style| {
            Button::new(
                state,
                Text::new(label)
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .size(16),
            )
            .width(Length::Fill)
            .padding(8)
            .on_press(message)
            .style(style)
        };

        let mut controls = Column::new()
            .spacing(5)
            .max_width(150)
            .push(button(
                split_horizontally,
                "Split horizontally",
                Message::Split(pane_grid::Axis::Horizontal, pane),
                style::Button::Primary,
            ))
            .push(button(
                split_vertically,
                "Split vertically",
                Message::Split(pane_grid::Axis::Vertical, pane),
                style::Button::Primary,
            ));

        if total_panes > 1 && !is_pinned {
            controls = controls.push(button(
                close,
                "Close",
                Message::Close(pane),
                style::Button::Destructive,
            ));
        }

        let content = Scrollable::new(scroll)
            .width(Length::Fill)
            .spacing(10)
            .align_items(Alignment::Center)
            .push(Text::new(format!("{}x{}", size.width, size.height)).size(24))
            .push(controls);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .center_y()
            .into()
    }
}

impl Controls {
    fn new() -> Self {
        Self {
            close: button::State::new(),
        }
    }

    pub fn view(
        &mut self,
        pane: pane_grid::Pane,
        total_panes: usize,
        is_pinned: bool,
    ) -> Element<Message> {
        let mut button = Button::new(&mut self.close, Text::new("Close").size(14))
            .style(style::Button::Control)
            .padding(3);
        if total_panes > 1 && !is_pinned {
            button = button.on_press(Message::Close(pane));
        }
        button.into()
    }
}
