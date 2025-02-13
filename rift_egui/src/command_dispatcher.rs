use std::collections::HashMap;

use egui::Ui;
use rift_core::{
    actions::{perform_action, Action},
    buffer::instance::Language,
    lsp::client::LSPClientHandle,
    state::{EditorState, Mode},
};

pub struct CommandDispatcher {}

impl CommandDispatcher {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &self,
        ui: &mut Ui,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ) {
        ui.input(|i| {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
            if !state.modal.open {
                for event in &i.raw.events {
                    state.update_view = true;
                    match event {
                        egui::Event::Text(text) => {
                            if matches!(state.mode, Mode::Insert) {
                                perform_action(
                                    Action::InsertTextAtCursor(text.to_string()),
                                    state,
                                    lsp_handle,
                                );
                            }
                        }
                        egui::Event::Key {
                            key,
                            physical_key: _,
                            pressed,
                            repeat: _,
                            modifiers,
                        } => {
                            if *pressed {
                                match key {
                                    egui::Key::Escape => {
                                        perform_action(Action::QuitInsertMode, state, lsp_handle);
                                    }
                                    egui::Key::I => {
                                        if matches!(state.mode, Mode::Normal) {
                                            perform_action(
                                                Action::EnterInsertMode,
                                                state,
                                                lsp_handle,
                                            );
                                            return;
                                        }
                                    }
                                    egui::Key::O => {
                                        if matches!(state.mode, Mode::Normal) {
                                            perform_action(
                                                Action::AddNewLineBelowAndEnterInsertMode,
                                                state,
                                                lsp_handle,
                                            );
                                            return;
                                        }
                                    }
                                    egui::Key::Comma => {
                                        if modifiers.shift {
                                            perform_action(Action::RemoveIndent, state, lsp_handle);
                                        } else {
                                            perform_action(
                                                Action::CyclePreviousBuffer,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::Period => {
                                        if modifiers.shift {
                                            perform_action(Action::AddIndent, state, lsp_handle);
                                        } else {
                                            perform_action(
                                                Action::CycleNextBuffer,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::Slash => {
                                        if modifiers.ctrl {
                                            perform_action(
                                                Action::CloseCurrentBuffer,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::X => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::SelectAndExtentCurrentLine,
                                                state,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::SelectCurrentLine,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::W => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::ExtendSelectTillEndOfWord,
                                                state,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::SelectTillEndOfWord,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::F => {
                                        if matches!(state.mode, Mode::Normal) {
                                            if modifiers.shift {
                                                rift_core::ai::ollama_fim(state);
                                            } else {
                                                perform_action(Action::OpenFile, state, lsp_handle);
                                                return;
                                            }
                                        }
                                    }
                                    egui::Key::B => {
                                        if matches!(state.mode, Mode::Normal) {
                                            perform_action(Action::SwitchBuffer, state, lsp_handle);
                                        }
                                    }
                                    egui::Key::S => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::SaveCurrentBuffer,
                                                state,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::FormatCurrentBuffer,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::ArrowDown => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::ExtendCursorDown,
                                                state,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::MoveCursorDown,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::ArrowUp => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::ExtendCursorUp,
                                                state,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(Action::MoveCursorUp, state, lsp_handle);
                                        }
                                    }
                                    egui::Key::ArrowLeft => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::ExtendCursorLeft,
                                                state,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::MoveCursorLeft,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::ArrowRight => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::ExtendCursorRight,
                                                state,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::MoveCursorRight,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::Home => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::ExtendCursorLineStart,
                                                state,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::MoveCursorLineStart,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::End => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::ExtendCursorLineEnd,
                                                state,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::MoveCursorLineEnd,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::G => {
                                        if !modifiers.shift {
                                            perform_action(
                                                Action::GoToBufferStart,
                                                state,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::GoToBufferEnd,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::Y => {
                                        if !modifiers.shift {
                                            perform_action(
                                                Action::CopyToRegister,
                                                state,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::CopyToClipboard,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::P => {
                                        if !modifiers.shift {
                                            perform_action(
                                                Action::PasteFromRegister,
                                                state,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::PasteFromClipboard,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::Semicolon => {
                                        perform_action(Action::Unselect, state, lsp_handle);
                                    }
                                    egui::Key::J => {
                                        if matches!(state.mode, Mode::Normal) {
                                            if modifiers.shift {
                                                perform_action(
                                                    Action::ExtendCursorDown,
                                                    state,
                                                    lsp_handle,
                                                );
                                            } else {
                                                perform_action(
                                                    Action::MoveCursorDown,
                                                    state,
                                                    lsp_handle,
                                                );
                                            }
                                        }
                                    }
                                    egui::Key::K => {
                                        if matches!(state.mode, Mode::Normal) {
                                            if modifiers.shift {
                                                perform_action(
                                                    Action::ExtendCursorUp,
                                                    state,
                                                    lsp_handle,
                                                );
                                            } else {
                                                perform_action(
                                                    Action::MoveCursorUp,
                                                    state,
                                                    lsp_handle,
                                                );
                                            }
                                        }
                                    }
                                    egui::Key::H => {
                                        if matches!(state.mode, Mode::Normal) {
                                            if modifiers.shift {
                                                perform_action(
                                                    Action::ExtendCursorLeft,
                                                    state,
                                                    lsp_handle,
                                                );
                                            } else {
                                                perform_action(
                                                    Action::MoveCursorLeft,
                                                    state,
                                                    lsp_handle,
                                                );
                                            }
                                        }
                                    }
                                    egui::Key::L => {
                                        if matches!(state.mode, Mode::Normal) {
                                            if modifiers.shift {
                                                perform_action(
                                                    Action::ExtendCursorRight,
                                                    state,
                                                    lsp_handle,
                                                );
                                            } else {
                                                perform_action(
                                                    Action::MoveCursorRight,
                                                    state,
                                                    lsp_handle,
                                                );
                                            }
                                        }
                                    }
                                    egui::Key::Z => {
                                        if !modifiers.shift {
                                            perform_action(Action::LSPHover, state, lsp_handle);
                                        } else {
                                            perform_action(
                                                Action::LSPCompletion,
                                                state,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::Backspace => {
                                        perform_action(
                                            Action::DeletePreviousCharacter,
                                            state,
                                            lsp_handle,
                                        );
                                    }
                                    egui::Key::Delete => {
                                        perform_action(
                                            Action::DeleteNextCharacter,
                                            state,
                                            lsp_handle,
                                        );
                                    }
                                    egui::Key::D => {
                                        perform_action(Action::DeleteSelection, state, lsp_handle);
                                    }
                                    egui::Key::Enter => {
                                        perform_action(
                                            Action::InsertNewLineAtCursor,
                                            state,
                                            lsp_handle,
                                        );
                                    }
                                    egui::Key::Tab => {
                                        perform_action(Action::AddTab, state, lsp_handle);
                                    }
                                    egui::Key::U => {
                                        if !modifiers.shift {
                                            perform_action(Action::Undo, state, lsp_handle);
                                        } else {
                                            perform_action(Action::Redo, state, lsp_handle);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                for event in &i.raw.events {
                    state.update_view = true;
                    match event {
                        egui::Event::Text(text) => {
                            let mut input = state.modal.input.clone();
                            input.push_str(text);
                            state.modal.set_input(input.clone());
                            if let Some(on_input) = state.modal.on_input {
                                state.modal.options = on_input(&input, state, lsp_handles);
                            }
                        }
                        egui::Event::Key {
                            key,
                            physical_key: _,
                            pressed,
                            repeat: _,
                            modifiers,
                        } => {
                            if *pressed {
                                match key {
                                    egui::Key::Tab => {
                                        state.modal.select_next();
                                    }
                                    egui::Key::Backspace => {
                                        let mut input = state.modal.input.clone();
                                        input.pop();
                                        state.modal.set_input(input.clone());
                                        if let Some(on_input) = state.modal.on_input {
                                            state.modal.options =
                                                on_input(&input, state, lsp_handles);
                                        }
                                    }
                                    egui::Key::Enter => {
                                        if let Some(on_select) = state.modal.on_select {
                                            if let Some(selection) = state.modal.selection {
                                                let alt = modifiers.shift;
                                                let options = state
                                                    .modal
                                                    .options
                                                    .get(selection)
                                                    .unwrap()
                                                    .clone();
                                                on_select(
                                                    state.modal.input.clone(),
                                                    &options,
                                                    alt,
                                                    state,
                                                    lsp_handles,
                                                );
                                            }
                                        }
                                    }
                                    egui::Key::Escape => {
                                        state.modal.close();
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}

impl Default for CommandDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
