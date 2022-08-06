use iced::Color;

// #[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Theme {
    pub surface: Color,
    pub active: Color,
    pub hovered: Color,
    pub pane_id_color_unfocused: Color,
    pub pane_id_color_focused: Color,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

pub const LIGHT_THEME: Theme = Theme {
    surface: rgb(0xF2, 0xF3, 0xF5),
    active: rgb(0x72, 0x89, 0xDA),
    hovered: rgb(0x67, 0x7B, 0xC4),
    pane_id_color_unfocused: rgb(0xFF, 0xC7, 0xC7),
    pane_id_color_focused: rgb(0xFF, 0x47, 0x47),
};

pub static THEME: Theme = LIGHT_THEME;
