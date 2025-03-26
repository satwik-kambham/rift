use std::collections::HashMap;

use rift_core::{buffer::instance::Language, lsp::client::LSPClientHandle, state::EditorState};

pub struct AIPanel {
    pub model_name: String,
}

impl AIPanel {
    pub fn new(model_name: String) -> Self {
        Self { model_name }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ) {
        egui::Window::new("AI Panel")
            .order(egui::Order::Foreground)
            .scroll(true)
            .show(ctx, |ui| {
                ui.label(&self.model_name);
                ui.text_edit_multiline(&mut self.model_name);
            });
    }
}

impl Default for AIPanel {
    fn default() -> Self {
        Self {
            model_name: "qwen2.5-coder:0.5b".into(),
        }
    }
}
