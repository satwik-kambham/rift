#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Preferences {
    pub line_ending: String,
    pub tab_width: usize,
    pub font_family: String,
    pub font_size: usize,
    pub line_height: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Theme {
    pub editor_bg: Color,
    pub cursor_selection_mode: Color,
    pub cursor_insert_mode: Color,
}

/// Color representation (values between 0 and 255)
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Color {
    pub r: usize,
    pub g: usize,
    pub b: usize,
    pub a: usize,
}

impl Color {
    pub fn from_rgb(r: usize, g: usize, b: usize) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn from_rgba(r: usize, g: usize, b: usize, a: usize) -> Self {
        Self { r, g, b, a }
    }
}
