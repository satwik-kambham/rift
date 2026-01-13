use egui::{FontId, RichText};
use rift_core::state::{EditorState, Mode};

pub fn show_status_line(ui: &mut egui::Ui, state: &mut EditorState) -> (f32, f32) {
    let mut char_width = 0.0;
    let mut char_height = 0.0;

    egui::Panel::bottom("status_line")
        .resizable(false)
        .show_separator_line(false)
        .frame(egui::Frame {
            fill: state.preferences.theme.status_bar_bg.into(),
            inner_margin: egui::Margin::symmetric(8, 8),
            ..Default::default()
        })
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let label_response = ui.label(
                    RichText::new("x")
                        .font(FontId::monospace(state.preferences.editor_font_size as f32)),
                );
                char_width = label_response.rect.width();
                char_height = label_response.rect.height();

                let mode = &state.mode;
                match mode {
                    Mode::Normal => ui.label(
                        RichText::new("NORMAL")
                            .color(state.preferences.theme.status_bar_normal_mode_fg),
                    ),
                    Mode::Insert => ui.label(
                        RichText::new("INSERT")
                            .color(state.preferences.theme.status_bar_insert_mode_fg),
                    ),
                };
                if state.audio_recording {
                    ui.separator();
                    ui.label(
                        RichText::new("⏺ REC").color(state.preferences.theme.error),
                    );
                }
                ui.separator();
                if state.buffer_idx.is_some() {
                    let (buffer, instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                    let file_path = buffer.display_name.clone();
                    let modified = buffer.modified;
                    let cursor = instance.cursor;

                    ui.label(
                        file_path
                            .as_ref()
                            .unwrap_or(&state.buffer_idx.unwrap().to_string()),
                    );
                    ui.separator();
                    ui.label(format!("{}:{}", cursor.row + 1, cursor.column + 1));
                    ui.separator();
                    ui.label(if modified { "U" } else { "" });
                    ui.separator();
                }
                ui.label(&state.keybind_handler.running_sequence);
                ui.separator();

                ui.label(state.log_messages.last().unwrap_or(&String::new()));

                ui.separator();
            });
        });
    (char_width, char_height)
}
