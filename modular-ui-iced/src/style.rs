use crate::theme::THEME;
use iced::{button, container, Background, Color, Vector};

pub enum TitleBar {
    Active,
    Focused,
}

impl container::StyleSheet for TitleBar {
    fn style(&self) -> container::Style {
        let pane = match self {
            Self::Active => Pane::Active,
            Self::Focused => Pane::Focused,
        }
        .style();

        container::Style {
            text_color: Some(Color::WHITE),
            background: Some(pane.border_color.into()),
            ..Default::default()
        }
    }
}

pub enum Pane {
    Active,
    Focused,
}

impl container::StyleSheet for Pane {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(THEME.surface)),
            border_width: 2.0,
            border_color: match self {
                Self::Active => Color::from_rgb(0.7, 0.7, 0.7),
                Self::Focused => Color::BLACK,
            },
            ..Default::default()
        }
    }
}

pub enum Button {
    Primary,
    Destructive,
    Control,
    Pin,
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        let (background, text_color) = match self {
            Button::Primary => (Some(THEME.active), Color::WHITE),
            Button::Destructive => (None, Color::from_rgb8(0xFF, 0x47, 0x47)),
            Button::Control => (Some(THEME.pane_id_color_focused), Color::WHITE),
            Button::Pin => (Some(THEME.active), Color::WHITE),
        };

        button::Style {
            text_color,
            background: background.map(Background::Color),
            border_radius: 3.0,
            shadow_offset: Vector::new(0.0, 0.0),
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();

        let background = match self {
            Button::Primary => Some(THEME.hovered),
            Button::Destructive => Some(Color {
                a: 0.2,
                ..active.text_color
            }),
            Button::Control => Some(THEME.pane_id_color_focused),
            Button::Pin => Some(THEME.hovered),
        };

        button::Style {
            background: background.map(Background::Color),
            ..active
        }
    }
}
