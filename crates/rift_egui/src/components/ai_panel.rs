use std::collections::HashMap;

use rift_core::{
    actions::{perform_action, Action},
    ai::ollama_fim,
    buffer::instance::Language,
    lsp::client::LSPClientHandle,
    state::EditorState,
};

#[derive(Default)]
pub struct AIPanel {}

impl AIPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ) {
        egui::Window::new("AI Panel")
            .order(egui::Order::Foreground)
            .default_open(false)
            .scroll(true)
            .show(ctx, |ui| {
                ui.collapsing(state.ai_state.model_name.clone(), |ui| {
                    ui.label("Model");
                    ui.text_edit_singleline(&mut state.ai_state.model_name);

                    ui.label("URL");
                    ui.text_edit_singleline(&mut state.ai_state.url);

                    ui.label("Seed");
                    ui.add(egui::DragValue::new(&mut state.ai_state.seed));

                    ui.label("Temperature");
                    ui.add(egui::DragValue::new(&mut state.ai_state.temperature));

                    ui.label("FIM Prompt");
                    ui.text_edit_multiline(state.ai_state.prompts.get_mut("fim").unwrap());
                });
                ui.text_edit_multiline(&mut state.ai_state.input);
                if ui.button(">").clicked() {
                    ollama_fim(state);
                }
                ui.separator();
                ui.label(&state.ai_state.output);
                ui.separator();
                if ui.button("accept").clicked() {
                    perform_action(
                        Action::InsertTextAtCursor(state.ai_state.output.clone()),
                        state,
                        lsp_handles,
                    );
                }
            });
    }
}
