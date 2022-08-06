use iced::Color;

// #[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Theme {
    pub surface: Color,
    pub active: Color,
    pub pane_active: Color,
    pub pane_focused: Color,
    pub hovered: Color,
    pub pane_id_color_unfocused: Color,
    pub pane_id_color_focused: Color,
    pub text_color: Color,
    pub text_color_disabled: Color,
    pub text_color_inv: Color,
    pub text_color_inv_disabled: Color,
    pub border_radius: f32,
    pub border_width: f32,
}

macro_rules! rgb {
    ($r:expr , $g:expr , $b:expr) => {
        Color::from_rgb($r as f32 / 255.0, $g as f32 / 255.0, $b as f32 / 255.0)
    };
}

pub const LIGHT_THEME: Theme = Theme {
    surface: rgb!(0xF2, 0xF3, 0xF5),
    active: rgb!(0x72, 0x89, 0xDA),
    pane_active: rgb!(0.7, 0.7, 0.7),
    pane_focused: rgb!(0, 0, 0),
    hovered: rgb!(0x67, 0x7B, 0xC4),
    pane_id_color_unfocused: rgb!(0xFF, 0xC7, 0xC7),
    pane_id_color_focused: rgb!(0xFF, 0x47, 0x47),
    text_color: rgb!(0x00, 0x00, 0x00),
    text_color_disabled: rgb!(0xFF, 0x47, 0x47),
    text_color_inv: rgb!(0xFF, 0xFF, 0xFF),
    text_color_inv_disabled: rgb!(0x00, 0x33, 0x33),
    border_radius: 4.0,
    border_width: 2.0,
};

pub static THEME: Theme = LIGHT_THEME;
