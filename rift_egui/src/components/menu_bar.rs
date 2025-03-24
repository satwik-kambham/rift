use std::collections::HashMap;

use rift_core::{
    actions::{perform_action, Action},
    buffer::instance::Language,
    lsp::client::LSPClientHandle,
    state::EditorState,
};

pub fn show_menu_bar(
    ctx: &egui::Context,
    state: &mut EditorState,
    lsp_handles: &mut HashMap<Language, LSPClientHandle>,
) {
    egui::TopBottomPanel::top("menu_bar")
        .resizable(false)
        .show_separator_line(true)
        .frame(egui::Frame {
            fill: state.preferences.theme.status_bar_bg.into(),
            inner_margin: egui::Margin::same(4.0),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.memory_mut(|mem| {
                if let Some(id) = mem.focused() {
                    mem.surrender_focus(id);
                }
            });
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open File / Folder").clicked() {
                        perform_action(Action::OpenFile, state, lsp_handles);
                    }
                    if ui.button("Save").clicked() {
                        perform_action(Action::SaveCurrentBuffer, state, lsp_handles);
                    }
                    if ui.button("Quit").clicked() {
                        perform_action(Action::Quit, state, lsp_handles);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        perform_action(Action::Undo, state, lsp_handles);
                    }
                    if ui.button("Redo").clicked() {
                        perform_action(Action::Redo, state, lsp_handles);
                    }
                    if ui.button("Unselect").clicked() {
                        perform_action(Action::Unselect, state, lsp_handles);
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Fuzzy Find File").clicked() {
                        perform_action(Action::FuzzyFindFile(false), state, lsp_handles);
                    }
                    if ui.button("Search Workspace").clicked() {
                        perform_action(Action::SearchWorkspace, state, lsp_handles);
                    }
                    if ui.button("Open Command Dispatcher").clicked() {
                        perform_action(Action::OpenCommandDispatcher, state, lsp_handles);
                    }
                });
                ui.menu_button("Navigation", |ui| {
                    if ui.button("Go To File Start").clicked() {
                        perform_action(Action::GoToBufferStart, state, lsp_handles);
                    }
                    if ui.button("Go To File End").clicked() {
                        perform_action(Action::GoToBufferEnd, state, lsp_handles);
                    }
                });
                ui.menu_button("LSP", |ui| {
                    if ui.button("Hover").clicked() {
                        perform_action(Action::LSPHover, state, lsp_handles);
                    }
                    if ui.button("Completion").clicked() {
                        perform_action(Action::LSPCompletion, state, lsp_handles);
                    }
                    if ui.button("Signature Help").clicked() {
                        perform_action(Action::LSPSignatureHelp, state, lsp_handles);
                    }
                    if ui.button("Go To Definition").clicked() {
                        perform_action(Action::GoToDefinition, state, lsp_handles);
                    }
                    if ui.button("Go To References").clicked() {
                        perform_action(Action::GoToReferences, state, lsp_handles);
                    }
                });
                ui.menu_button("Preferences", |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("+").clicked() {
                            state.preferences.editor_font_size += 1;
                        };
                        ui.label(format!("Font Size: {}", state.preferences.editor_font_size));
                        if ui.button("-").clicked() {
                            state.preferences.editor_font_size -= 1;
                        };
                    });
                    if ui
                        .button(format!("Tab Size: {}", state.preferences.tab_width))
                        .clicked()
                    {
                        if state.preferences.tab_width == 4 {
                            state.preferences.tab_width = 2;
                        } else {
                            state.preferences.tab_width = 4;
                        }
                    };
                    if ui
                        .button(
                            (if state.preferences.line_ending == "\n" {
                                "lf"
                            } else {
                                "crlf"
                            })
                            .to_string(),
                        )
                        .clicked()
                    {
                        if state.preferences.line_ending == "\n" {
                            state.preferences.line_ending = "\r\n".to_string()
                        } else {
                            state.preferences.line_ending = "\n".to_string();
                        }
                    };
                    ui.checkbox(
                        &mut state.preferences.trigger_completion_on_type,
                        "Trigger Completions",
                    );
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("Keybind Help").clicked() {
                        perform_action(Action::KeybindHelp, state, lsp_handles);
                    }
                });
            });
        });
}
