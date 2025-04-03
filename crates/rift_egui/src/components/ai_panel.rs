use std::collections::HashMap;

use rift_core::{
    actions::{perform_action, Action},
    ai::{ollama_chat, ollama_fim},
    buffer::instance::Language,
    lsp::client::LSPClientHandle,
    state::EditorState,
};

#[derive(Default, PartialEq)]
pub enum PanelType {
    FIM,
    #[default]
    Chat,
}

#[derive(Default)]
pub struct AIPanel {
    pub panel_type: PanelType,
}

impl AIPanel {
    pub fn new() -> Self {
        Self {
            panel_type: PanelType::Chat,
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ) {
        if state.preferences.show_ai_panel {
            egui::Window::new("AI Panel")
                .order(egui::Order::Foreground)
                .scroll(true)
                .show(ctx, |ui| {
                    ui.selectable_value(&mut self.panel_type, PanelType::FIM, "FIM");
                    ui.selectable_value(&mut self.panel_type, PanelType::Chat, "Chat");

                    match self.panel_type {
                        PanelType::FIM => {
                            ui.collapsing("Options", |ui| {
                                ui.label("Model");
                                ui.text_edit_singleline(
                                    &mut state.ai_state.generate_state.model_name,
                                );

                                ui.label("URL");
                                ui.text_edit_singleline(&mut state.ai_state.generate_state.url);

                                ui.label("Seed");
                                ui.add(egui::DragValue::new(
                                    &mut state.ai_state.generate_state.seed,
                                ));

                                ui.label("Temperature");
                                ui.add(egui::DragValue::new(
                                    &mut state.ai_state.generate_state.temperature,
                                ));

                                ui.label("FIM Prompt");
                                ui.text_edit_multiline(
                                    state
                                        .ai_state
                                        .generate_state
                                        .prompts
                                        .get_mut("fim")
                                        .unwrap(),
                                );
                            });
                            ui.text_edit_multiline(&mut state.ai_state.generate_state.input);
                            if ui.button(">").clicked() {
                                ollama_fim(state);
                            }
                            ui.separator();
                            ui.label(&state.ai_state.generate_state.output);
                            ui.separator();
                            if ui.button("accept").clicked() {
                                perform_action(
                                    Action::InsertTextAtCursor(
                                        state.ai_state.generate_state.output.clone(),
                                    ),
                                    state,
                                    lsp_handles,
                                );
                            }
                        }
                        PanelType::Chat => {
                            ui.collapsing("Options", |ui| {
                                ui.label("Model");
                                ui.text_edit_singleline(&mut state.ai_state.chat_state.model_name);

                                ui.label("URL");
                                ui.text_edit_singleline(&mut state.ai_state.chat_state.url);

                                ui.label("Seed");
                                ui.add(egui::DragValue::new(&mut state.ai_state.chat_state.seed));

                                ui.label("Temperature");
                                ui.add(egui::DragValue::new(
                                    &mut state.ai_state.chat_state.temperature,
                                ));
                            });
                            ui.text_edit_multiline(&mut state.ai_state.chat_state.input);
                            if ui.button(">").clicked() {
                                ollama_chat(state);
                            }
                            ui.separator();
                            for message in &state.ai_state.chat_state.history {
                                ui.label(egui::RichText::new(&message.role).strong());
                                ui.label(&message.content);
                                ui.separator();
                            }
                            ui.separator();
                            if ui.button("clear").clicked() {
                                state.ai_state.chat_state.history.clear();
                            }
                        }
                    }
                });
        }
    }
}
