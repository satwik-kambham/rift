use crate::buffer::instance::Language;
use crate::themes;

/// Color representation (values between 0 and 255)
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
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
    pub ui_font_size_heading: usize,
    pub ui_font_size_button: usize,
    pub ui_font_size_small: usize,
    pub line_height: f32,
    pub gutter_padding: i8,
    pub editor_padding: i8,
    pub trigger_completion_on_type: bool,
    pub show_file_explorer: bool,
    pub show_ai_panel: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        let line_ending = if cfg!(target_os = "windows") {
            String::from("\r\n")
        } else {
            String::from("\n")
        };

        Self {
            theme: Theme::catppuccin_kanagawa(),
            line_ending,
            tab_width: 4,
            editor_font_family: "Monaspace Neon".into(),
            editor_font_size: 16,
            ui_font_family: "Open Sans".into(),
            ui_font_size: 14,
            ui_font_size_heading: 16,
            ui_font_size_button: 14,
            ui_font_size_small: 12,
            line_height: 1.5,
            gutter_padding: 4,
            editor_padding: 4,
            trigger_completion_on_type: true,
            show_file_explorer: false,
            show_ai_panel: false,
        }
    }
}

impl Preferences {
    pub fn get_comment_token(language: Language) -> String {
        match language {
            Language::RSL | Language::Python | Language::TOML | Language::Nix => "# ",
            Language::Rust
            | Language::Dart
            | Language::Javascript
            | Language::Typescript
            | Language::C
            | Language::CPP => "// ",
            _ => "",
        }
        .to_string()
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Theme {
    pub name: String,
    pub editor_bg: Color,
    pub selection_bg: Color,
    pub ui_border: Color,
    pub cursor_normal_mode_fg: Color,
    pub cursor_insert_mode_fg: Color,
    pub cursor_normal_mode_bg: Color,
    pub cursor_insert_mode_bg: Color,
    pub status_bar_bg: Color,
    pub status_bar_normal_mode_fg: Color,
    pub status_bar_insert_mode_fg: Color,
    pub gutter_bg: Color,
    pub gutter_text: Color,
    pub gutter_text_current_line: Color,
    pub modal_bg: Color,
    pub modal_text: Color,
    pub modal_active: Color,
    pub modal_primary: Color,
    pub highlight_none: Color,
    pub highlight_white: Color,
    pub highlight_red: Color,
    pub highlight_orange: Color,
    pub highlight_blue: Color,
    pub highlight_green: Color,
    pub highlight_purple: Color,
    pub highlight_yellow: Color,
    pub highlight_gray: Color,
    pub highlight_turquoise: Color,
    pub ui_text: Color,
    pub ui_bg_fill: Color,
    pub ui_weak_bg_fill: Color,
    pub ui_bg_stroke: Color,
    pub ui_fg_stroke: Color,
    pub error: Color,
    pub warning: Color,
    pub information: Color,
    pub hint: Color,
}

#[allow(dead_code)]
impl Theme {
    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "Catppuccin Mocha".into(),
            editor_bg: themes::catppuccin_mocha::BASE,
            selection_bg: themes::catppuccin_mocha::SURFACE2,
            ui_border: themes::catppuccin_mocha::CRUST,
            cursor_normal_mode_fg: themes::catppuccin_mocha::BASE,
            cursor_insert_mode_fg: themes::catppuccin_mocha::BASE,
            cursor_normal_mode_bg: themes::catppuccin_mocha::MAUVE,
            cursor_insert_mode_bg: themes::catppuccin_mocha::GREEN,
            status_bar_bg: themes::catppuccin_mocha::BASE,
            status_bar_normal_mode_fg: themes::catppuccin_mocha::MAUVE,
            status_bar_insert_mode_fg: themes::catppuccin_mocha::GREEN,
            gutter_bg: themes::catppuccin_mocha::BASE,
            gutter_text: themes::catppuccin_mocha::SUBTEXT0,
            gutter_text_current_line: themes::catppuccin_mocha::MAUVE,
            highlight_none: themes::catppuccin_mocha::TEXT,
            highlight_white: themes::catppuccin_mocha::TEXT,
            highlight_red: themes::catppuccin_mocha::RED,
            highlight_orange: themes::catppuccin_mocha::PEACH,
            highlight_blue: themes::catppuccin_mocha::BLUE,
            highlight_green: themes::catppuccin_mocha::GREEN,
            highlight_purple: themes::catppuccin_mocha::MAUVE,
            highlight_yellow: themes::catppuccin_mocha::YELLOW,
            highlight_gray: themes::catppuccin_mocha::OVERLAY0,
            highlight_turquoise: themes::catppuccin_mocha::TEAL,
            modal_bg: themes::catppuccin_mocha::MANTLE,
            modal_text: themes::catppuccin_mocha::TEXT,
            modal_active: themes::catppuccin_mocha::SUBTEXT0,
            modal_primary: themes::catppuccin_mocha::MAUVE,
            ui_text: themes::catppuccin_mocha::TEXT,
            ui_bg_fill: themes::catppuccin_mocha::SURFACE1,
            ui_weak_bg_fill: themes::catppuccin_mocha::SURFACE0,
            ui_bg_stroke: themes::catppuccin_mocha::OVERLAY1,
            ui_fg_stroke: themes::catppuccin_mocha::OVERLAY2,
            error: themes::catppuccin_mocha::RED,
            warning: themes::catppuccin_mocha::MAROON,
            information: themes::catppuccin_mocha::YELLOW,
            hint: themes::catppuccin_mocha::BLUE,
        }
    }

    pub fn onedark() -> Self {
        Self {
            name: "One Dark".into(),
            editor_bg: themes::onedark::SYNTAX_BG,
            selection_bg: themes::onedark::SYNTAX_SELECTION,
            ui_border: themes::onedark::UI_BORDER,
            cursor_normal_mode_fg: themes::onedark::SYNTAX_BG,
            cursor_insert_mode_fg: themes::onedark::SYNTAX_BG,
            cursor_normal_mode_bg: themes::onedark::BLUE,
            cursor_insert_mode_bg: themes::onedark::GREEN,
            status_bar_bg: themes::onedark::UI_BG,
            status_bar_normal_mode_fg: themes::onedark::BLUE,
            status_bar_insert_mode_fg: themes::onedark::GREEN,
            gutter_bg: themes::onedark::SYNTAX_BG,
            gutter_text: themes::onedark::SYNTAX_GUTTER,
            gutter_text_current_line: themes::onedark::MONO1,
            highlight_none: themes::onedark::MONO1,
            highlight_white: themes::onedark::MONO1,
            highlight_red: themes::onedark::RED1,
            highlight_orange: themes::onedark::ORANGE2,
            highlight_blue: themes::onedark::BLUE,
            highlight_green: themes::onedark::GREEN,
            highlight_purple: themes::onedark::PURPLE,
            highlight_yellow: themes::onedark::ORANGE1,
            highlight_gray: themes::onedark::MONO2,
            highlight_turquoise: themes::onedark::CYAN,
            modal_bg: themes::onedark::UI_BG,
            modal_text: themes::onedark::MONO2,
            modal_active: themes::onedark::MONO1,
            modal_primary: themes::onedark::BLUE,
            ui_text: themes::onedark::MONO1,
            ui_bg_fill: themes::onedark::SYNTAX_BG,
            ui_weak_bg_fill: themes::onedark::SYNTAX_BG,
            ui_bg_stroke: themes::onedark::MONO3,
            ui_fg_stroke: themes::onedark::MONO2,
            error: themes::onedark::RED1,
            warning: themes::onedark::ORANGE1,
            information: themes::onedark::CYAN,
            hint: themes::onedark::BLUE,
        }
    }

    pub fn kanagawa() -> Self {
        Self {
            name: "Kanagawa".into(),
            editor_bg: themes::kanagawa::BLACK3,
            selection_bg: themes::kanagawa::BLACK5,
            ui_border: themes::kanagawa::BLACK0,
            cursor_normal_mode_fg: themes::kanagawa::BLACK3,
            cursor_insert_mode_fg: themes::kanagawa::BLACK3,
            cursor_normal_mode_bg: themes::kanagawa::BLUE,
            cursor_insert_mode_bg: themes::kanagawa::GREEN1,
            status_bar_bg: themes::kanagawa::BLACK0,
            status_bar_normal_mode_fg: themes::kanagawa::BLUE,
            status_bar_insert_mode_fg: themes::kanagawa::GREEN1,
            gutter_bg: themes::kanagawa::BLACK3,
            gutter_text: themes::kanagawa::GRAY2,
            gutter_text_current_line: themes::kanagawa::WHITE1,
            highlight_none: themes::kanagawa::WHITE0,
            highlight_white: themes::kanagawa::WHITE0,
            highlight_red: themes::kanagawa::RED,
            highlight_orange: themes::kanagawa::ORANGE0,
            highlight_blue: themes::kanagawa::BLUE,
            highlight_green: themes::kanagawa::GREEN0,
            highlight_purple: themes::kanagawa::VIOLET,
            highlight_yellow: themes::kanagawa::YELLOW,
            highlight_gray: themes::kanagawa::GRAY0,
            highlight_turquoise: themes::kanagawa::TEAL,
            modal_bg: themes::kanagawa::BLACK0,
            modal_text: themes::kanagawa::WHITE0,
            modal_active: themes::kanagawa::WHITE1,
            modal_primary: themes::kanagawa::BLUE,
            ui_text: themes::kanagawa::WHITE0,
            ui_bg_fill: themes::kanagawa::BLACK3,
            ui_weak_bg_fill: themes::kanagawa::BLACK1,
            ui_bg_stroke: themes::kanagawa::BLACK1,
            ui_fg_stroke: themes::kanagawa::GRAY2,
            error: themes::kanagawa::RED,
            warning: themes::kanagawa::ORANGE1,
            information: themes::kanagawa::TEAL,
            hint: themes::kanagawa::BLUE,
        }
    }
    pub fn catppuccin_kanagawa() -> Self {
        Self {
            name: "Catppuccin Kanagawa".into(),
            editor_bg: themes::kanagawa::BLACK3,
            selection_bg: themes::kanagawa::BLACK5,
            ui_border: themes::kanagawa::BLACK0,
            cursor_normal_mode_fg: themes::catppuccin_mocha::BASE,
            cursor_insert_mode_fg: themes::catppuccin_mocha::BASE,
            cursor_normal_mode_bg: themes::catppuccin_mocha::MAUVE,
            cursor_insert_mode_bg: themes::catppuccin_mocha::GREEN,
            status_bar_bg: themes::kanagawa::BLACK0,
            status_bar_normal_mode_fg: themes::catppuccin_mocha::MAUVE,
            status_bar_insert_mode_fg: themes::catppuccin_mocha::GREEN,
            gutter_bg: themes::kanagawa::BLACK3,
            gutter_text: themes::catppuccin_mocha::SUBTEXT0,
            gutter_text_current_line: themes::catppuccin_mocha::MAUVE,
            highlight_none: themes::catppuccin_mocha::TEXT,
            highlight_white: themes::catppuccin_mocha::TEXT,
            highlight_red: themes::catppuccin_mocha::RED,
            highlight_orange: themes::catppuccin_mocha::PEACH,
            highlight_blue: themes::catppuccin_mocha::BLUE,
            highlight_green: themes::catppuccin_mocha::GREEN,
            highlight_purple: themes::catppuccin_mocha::MAUVE,
            highlight_yellow: themes::catppuccin_mocha::YELLOW,
            highlight_gray: themes::catppuccin_mocha::OVERLAY0,
            highlight_turquoise: themes::catppuccin_mocha::TEAL,
            modal_bg: themes::kanagawa::BLACK0,
            modal_text: themes::catppuccin_mocha::TEXT,
            modal_active: themes::catppuccin_mocha::SUBTEXT0,
            modal_primary: themes::catppuccin_mocha::MAUVE,
            ui_text: themes::catppuccin_mocha::TEXT,
            ui_bg_fill: themes::kanagawa::BLACK3,
            ui_weak_bg_fill: themes::kanagawa::BLACK1,
            ui_bg_stroke: themes::kanagawa::BLACK1,
            ui_fg_stroke: themes::kanagawa::GRAY2,
            error: themes::catppuccin_mocha::RED,
            warning: themes::catppuccin_mocha::MAROON,
            information: themes::catppuccin_mocha::YELLOW,
            hint: themes::catppuccin_mocha::BLUE,
        }
    }
}
