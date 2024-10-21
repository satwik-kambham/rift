use crate::themes;

/// Color representation (values between 0 and 255)
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Color> for ecolor::Color32 {
    fn from(val: Color) -> Self {
        ecolor::Color32::from_rgb(val.r, val.g, val.b)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Preferences {
    pub theme: Theme,
    pub line_ending: String,
    pub tab_width: usize,
    pub editor_font_family: String,
    pub ui_font_family: String,
    pub editor_font_size: usize,
    pub ui_font_size: usize,
    pub line_height: f32,
    pub gutter_padding: f32,
    pub editor_padding: f32,
}

impl Default for Preferences {
    fn default() -> Self {
        let line_ending = if cfg!(target_os = "windows") {
            String::from("\r\n")
        } else {
            String::from("\n")
        };

        Self {
            theme: Theme::catppuccin_mocha(),
            line_ending,
            tab_width: 4,
            editor_font_family: "".into(),
            editor_font_size: 18,
            ui_font_family: "".into(),
            ui_font_size: 18,
            line_height: 1.5,
            gutter_padding: 8.0,
            editor_padding: 8.0,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Theme {
    pub editor_bg: Color,
    pub cursor_normal_mode_fg: Color,
    pub cursor_insert_mode_fg: Color,
    pub cursor_normal_mode_bg: Color,
    pub cursor_insert_mode_bg: Color,
    pub status_bar_bg: Color,
    pub status_bar_normal_mode_fg: Color,
    pub status_bar_insert_mode_fg: Color,
    pub status_bar_normal_mode_bg: Color,
    pub status_bar_insert_mode_bg: Color,
    pub gutter_bg: Color,
    pub gutter_text: Color,
    pub gutter_text_current_line: Color,
}

impl Theme {
    fn catppuccin_mocha() -> Self {
        Self {
            editor_bg: themes::catppuccin_mocha::BASE,
            cursor_normal_mode_fg: themes::catppuccin_mocha::BASE,
            cursor_insert_mode_fg: themes::catppuccin_mocha::BASE,
            cursor_normal_mode_bg: themes::catppuccin_mocha::MAUVE,
            cursor_insert_mode_bg: themes::catppuccin_mocha::GREEN,
            status_bar_bg: themes::catppuccin_mocha::BASE,
            status_bar_normal_mode_fg: themes::catppuccin_mocha::BASE,
            status_bar_insert_mode_fg: themes::catppuccin_mocha::BASE,
            status_bar_normal_mode_bg: themes::catppuccin_mocha::MAUVE,
            status_bar_insert_mode_bg: themes::catppuccin_mocha::GREEN,
            gutter_bg: themes::catppuccin_mocha::BASE,
            gutter_text: themes::catppuccin_mocha::SUBTEXT0,
            gutter_text_current_line: themes::catppuccin_mocha::MAUVE,
        }
    }
}
