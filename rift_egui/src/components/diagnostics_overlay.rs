pub struct DiagnosticsOverlay {
    pub info: String,
}

impl DiagnosticsOverlay {
    pub fn new() -> Self {
        Self {
            info: "".to_string(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::Window::new("diagnostics_overlay")
            .movable(false)
            .interactable(true)
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::ZERO)
            .resizable(false)
            .collapsible(true)
            .title_bar(true)
            .vscroll(true)
            .frame(egui::Frame {
                fill: egui::Color32::TRANSPARENT,
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.label(&self.info);
            });
    }
}

impl Default for DiagnosticsOverlay {
    fn default() -> Self {
        Self::new()
    }
}
