pub struct InfoModal {
    pub info: String,
    pub active: bool,
}

impl InfoModal {
    pub fn new() -> Self {
        Self {
            info: "".to_string(),
            active: false,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        if self.active {
            egui::Window::new("info_modal")
                .movable(false)
                .interactable(true)
                .order(egui::Order::Tooltip)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .resizable(false)
                .collapsible(false)
                .title_bar(false)
                .vscroll(true)
                .show(ctx, |ui| {
                    ui.label(&self.info);
                    self.handle_input(ui);
                });
            return false;
        }
        true
    }

    pub fn handle_input(&mut self, ui: &mut egui::Ui) {
        ui.input(|i| {
            for event in &i.raw.events {
                if let egui::Event::Key {
                    key,
                    physical_key: _,
                    pressed,
                    repeat: _,
                    modifiers: _,
                } = event
                {
                    if *pressed {
                        match key {
                            egui::Key::Escape => {
                                self.active = false;
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
    }
}

impl Default for InfoModal {
    fn default() -> Self {
        Self::new()
    }
}
