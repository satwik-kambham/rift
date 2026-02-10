use rift_core::preferences::Color;

pub(crate) fn color_from_rgb(c: Color) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(c.r, c.g, c.b)
}
