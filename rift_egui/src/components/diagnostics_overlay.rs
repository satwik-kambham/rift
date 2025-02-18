use rift_core::state::EditorState;

pub fn show_diagnostics_overlay(ctx: &egui::Context, state: &EditorState) {
    egui::Window::new("diagnostics_overlay")
        .movable(false)
        .interactable(true)
        .order(egui::Order::Tooltip)
        .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::ZERO)
        .resizable(false)
        .collapsible(true)
        .title_bar(false)
        .vscroll(true)
        .frame(egui::Frame {
            fill: egui::Color32::TRANSPARENT,
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.label(&state.diagnostics_overlay.content);
        });
}
