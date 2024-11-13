pub struct InfoModal {
    pub info: String,
}

impl InfoModal {
    pub fn new() -> Self {
        Self {
            info: "".to_string(),
        }
    }

    pub fn show(&self, ctx: &egui::Context) {
        egui::Window::new("info_modal")
            .movable(false)
            .interactable(false)
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.label(&self.info);
            });
    }
}

impl Default for InfoModal {
    fn default() -> Self {
        Self::new()
    }
}
