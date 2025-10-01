use std::collections::HashMap;

use rift_core::{
    actions::{perform_action, Action},
    ai::{llamacpp_chat, ollama_chat, ollama_fim, openrouter_chat, ChatState},
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
        floating: bool,
    ) {
        if state.preferences.show_ai_panel {
            if floating {
                egui::Window::new("AI Panel")
                    .order(egui::Order::Foreground)
                    .scroll(true)
                    .show(ctx, |ui| {
                        self.draw(ui, state, lsp_handles);
                    });
            } else {
                egui::SidePanel::right("AI Side Panel")
                    .resizable(true)
                    .frame(egui::Frame {
                        fill: state.preferences.theme.gutter_bg.into(),
                        inner_margin: egui::Margin::same(state.preferences.gutter_padding),
                        ..Default::default()
                    })
                    .show(ctx, |ui| {
                        self.draw(ui, state, lsp_handles);
                    });
            }
        }
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.panel_type, PanelType::FIM, "FIM");
            ui.selectable_value(&mut self.panel_type, PanelType::Chat, "Chat");
        });
        match self.panel_type {
            PanelType::FIM => {
                ui.collapsing("Options", |ui| {
                    ui.label("Model");
                    ui.text_edit_singleline(&mut state.ai_state.generate_state.model_name);

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
                        Action::InsertTextAtCursor(state.ai_state.generate_state.output.clone()),
                        state,
                        lsp_handles,
                    );
                }
            }
            PanelType::Chat => {
                ui.horizontal(|ui| {
                    if ui.button("llamacpp").clicked() {
                        state.ai_state.chat_state = ChatState::llamacpp();
                    }
                    if ui.button("ollama").clicked() {
                        state.ai_state.chat_state = ChatState::ollama();
                    }
                    if ui.button("openrouter").clicked() {
                        state.ai_state.chat_state = ChatState::openrouter();
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Default").clicked() {
                        perform_action(
                            Action::SetSystemPrompt("default".to_string()),
                            state,
                            lsp_handles,
                        );
                    }
                    if ui.button("Agentic Coding").clicked() {
                        perform_action(
                            Action::SetSystemPrompt("agentic_coding".to_string()),
                            state,
                            lsp_handles,
                        );
                    }
                });

                ui.checkbox(
                    &mut state.ai_state.full_user_control,
                    "Enable full user control",
                );

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
                    ui.label(message.content.clone().unwrap_or_default());
                    if let Some(tool_calls) = &message.tool_calls {
                        ui.label(format!("Tool Requested: {}", tool_calls));
                    }
                    ui.separator();
                }
                ui.separator();
                if !state.ai_state.pending_tool_calls.is_empty() {
                    for (tool_name, _, _, tool_preview) in &state.ai_state.pending_tool_calls {
                        ui.label(tool_name);
                        ui.label(tool_preview);
                    }
                    if ui.button("approve").clicked() {
                        let (tool_name, tool_args, tool_call_id, _) =
                            state.ai_state.pending_tool_calls.remove(0);
                        rift_core::ai::tool_calling::handle_tool_calls(
                            tool_name,
                            tool_args,
                            tool_call_id,
                            state,
                            true,
                        );
                    }
                    if ui.button("deny").clicked() {
                        let (tool_name, tool_args, tool_call_id, _) =
                            state.ai_state.pending_tool_calls.remove(0);
                        rift_core::ai::tool_calling::handle_tool_calls(
                            tool_name,
                            tool_args,
                            tool_call_id,
                            state,
                            false,
                        );
                    }
                }
                ui.separator();
                ui.text_edit_multiline(&mut state.ai_state.chat_state.input);
                if ui.button(">").clicked() {
                    if state.ai_state.chat_state.provider == "llamacpp" {
                        llamacpp_chat(state);
                    } else if state.ai_state.chat_state.provider == "ollama" {
                        ollama_chat(state);
                    } else if state.ai_state.chat_state.provider == "openrouter" {
                        openrouter_chat(state);
                    }
                    state.ai_state.chat_state.input.clear();
                }
                ui.separator();

                if ui.button("clear").clicked() {
                    state.ai_state.chat_state.history.clear();
                    state.ai_state.pending_tool_calls.clear();
                }
            }
        }
    }
}
