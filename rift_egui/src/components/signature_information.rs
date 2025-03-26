use rift_core::state::EditorState;

pub fn show_signature_information(
    char_width: f32,
    char_height: f32,
    top_left: egui::Pos2,
    ctx: &egui::Context,
    state: &EditorState,
) {
    let offset = egui::Pos2 {
        x: (state.relative_cursor.column as f32 * char_width)
            + top_left.x
            + char_width
            + state.preferences.editor_padding as f32,
        y: ((state.relative_cursor.row - 1) as f32 * char_height)
            + top_left.y
            + char_height
            + state.preferences.editor_padding as f32,
    };
    egui::Window::new("signature_information")
        .movable(false)
        .interactable(true)
        .order(egui::Order::Tooltip)
        .pivot(egui::Align2::LEFT_BOTTOM)
        .fixed_pos(offset)
        .resizable(false)
        .auto_sized()
        .collapsible(true)
        .title_bar(false)
        .vscroll(true)
        .show(ctx, |ui| {
            ui.label(&state.signature_information.content);
        });
}
