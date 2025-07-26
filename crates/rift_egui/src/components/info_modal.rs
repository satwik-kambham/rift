use rift_core::state::EditorState;

pub struct InfoModalWidget {}

impl InfoModalWidget {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&self, ctx: &egui::Context, state: &mut EditorState) {
        if state.info_modal.active {
            egui::Window::new("info_modal")
                .movable(false)
                .interactable(true)
                .order(egui::Order::Tooltip)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .resizable(true)
                .collapsible(false)
                .title_bar(false)
                .vscroll(true)
                .show(ctx, |ui| {
                    ui.label(&state.info_modal.content);
                    self.handle_input(ui, state);
                });
        }
    }

    pub fn handle_input(&self, ui: &mut egui::Ui, state: &mut EditorState) {
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
                    if *pressed && key == &egui::Key::Escape {
                        state.info_modal.close();
                    }
                }
            }
        });
    }
}

impl Default for InfoModalWidget {
    fn default() -> Self {
        Self::new()
    }
}
