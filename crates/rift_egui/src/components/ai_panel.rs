use std::collections::HashMap;

use rift_core::{
    actions::{perform_action, Action},
    ai::{ollama_chat, ollama_fim, openrouter_chat, ChatState},
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

                                ui.label("Context Length");
                                ui.add(egui::DragValue::new(
                                    &mut state.ai_state.generate_state.num_ctx,
                                ));

                                ui.label("FIM Prompt");
                                ui.text_edit_multiline(
                                    state
                                        .ai_state
                                        .generate_state
                                        .prompts
                                        .get_mut("file_fim")
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
                            if ui.button("ollama").clicked() {
                                state.ai_state.chat_state = ChatState::ollama();
                            }
                            if ui.button("openrouter").clicked() {
                                state.ai_state.chat_state = ChatState::openrouter();
                            }

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
                            for message in &state.ai_state.chat_state.history {
                                ui.label(egui::RichText::new(&message.role).strong());
                                ui.label(&message.content);
                                if let Some(tool_calls) = &message.tool_calls {
                                    ui.label(format!("Tool Requested: {}", tool_calls.to_string()));
                                }
                                ui.separator();
                            }
                            ui.separator();
                            ui.text_edit_multiline(&mut state.ai_state.chat_state.input);
                            if ui.button(">").clicked() {
                                if state.ai_state.chat_state.provider == "ollama" {
                                    ollama_chat(state);
                                } else if state.ai_state.chat_state.provider == "openrouter" {
                                    openrouter_chat(state);
                                }
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
