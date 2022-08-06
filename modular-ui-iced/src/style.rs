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
            text_color: Some(THEME.text_color_inv),
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
            border_width: THEME.border_width,
            border_color: match self {
                Self::Active => THEME.pane_active,
                Self::Focused => THEME.pane_focused,
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
            Button::Primary => (Some(THEME.active), THEME.text_color_inv),
            Button::Destructive => (None, THEME.text_color_inv_disabled),
            Button::Control => (Some(THEME.pane_id_color_focused), THEME.text_color_inv),
            Button::Pin => (Some(THEME.active), THEME.text_color_inv),
        };

        button::Style {
            text_color,
            background: background.map(Background::Color),
            border_radius: THEME.border_radius,
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
