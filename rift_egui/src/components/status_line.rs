use egui::RichText;
use rift_core::state::{EditorState, Mode};

pub fn show_status_line(ctx: &egui::Context, state: &mut EditorState) {
    egui::TopBottomPanel::bottom("status_line")
        .resizable(false)
        .show_separator_line(false)
        .frame(egui::Frame {
            fill: state.preferences.theme.status_bar_bg.into(),
            inner_margin: egui::Margin::symmetric(8.0, 8.0),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.memory_mut(|mem| {
                if let Some(id) = mem.focused() {
                    mem.surrender_focus(id);
                }
            });
            if state.buffer_idx.is_some() {
                let (buffer, instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                let file_path = buffer.file_path.clone();
                let modified = buffer.modified;
                let cursor = instance.cursor;

                ui.horizontal(|ui| {
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
                    ui.separator();
                    ui.label(file_path.as_ref().unwrap());
                    ui.separator();
                    ui.label(format!("{}:{}", cursor.row + 1, cursor.column + 1));
                    ui.separator();
                    ui.label(if modified { "U" } else { "" });
                    ui.separator();
                });
            }
        });
}
