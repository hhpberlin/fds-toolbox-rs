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
    pub title_bar_size: u16,
    pub pane_padding: u16,
    pub pane_padding_grabbable: u16,
    pub window_padding: u16,
}

macro_rules! rgb {
    ($r:expr , $g:expr , $b:expr , u8) => {
        Color::from_rgb($r as f32 / 255.0, $g as f32 / 255.0, $b as f32 / 255.0)
    };
    ($r:expr , $g:expr , $b:expr , f32) => {
        Color::from_rgb($r as f32, $g as f32, $b as f32)
    };
}

pub const LIGHT_THEME: Theme = Theme {
    surface: rgb!(0xF2, 0xF3, 0xF5, u8),
    active: rgb!(0x72, 0x89, 0xDA, u8),
    pane_active: rgb!(0.7, 0.7, 0.7, f32),
    pane_focused: rgb!(0, 0, 0, f32),
    hovered: rgb!(0x67, 0x7B, 0xC4, u8),
    pane_id_color_unfocused: rgb!(0xFF, 0xC7, 0xC7, u8),
    pane_id_color_focused: rgb!(0xFF, 0x47, 0x47, u8),
    text_color: rgb!(0x00, 0x00, 0x00, u8),
    text_color_disabled: rgb!(0xFF, 0x47, 0x47, u8),
    text_color_inv: rgb!(0xFF, 0xFF, 0xFF, u8),
    text_color_inv_disabled: rgb!(0x00, 0x33, 0x33, u8),
    border_radius: 4.0,
    border_width: 2.0,
    title_bar_size: 4,
    pane_padding: 6,
    pane_padding_grabbable: 6,
    window_padding: 6,
};

pub static THEME: Theme = LIGHT_THEME;
