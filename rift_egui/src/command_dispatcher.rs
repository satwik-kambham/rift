use egui::Ui;
use rift_core::{
    actions::{perform_action, Action},
    buffer::line_buffer::LineBuffer,
    io::file_io,
    lsp::client::LSPClientHandle,
    preferences::Preferences,
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
        preferences: &mut Preferences,
        lsp_handle: &mut LSPClientHandle,
    ) {
        ui.input(|i| {
            if !state.modal_open {
                for event in &i.raw.events {
                    state.update_view = true;
                    match event {
                        egui::Event::Text(text) => {
                            perform_action(
                                Action::InsertTextAtCursor(text.to_string()),
                                state,
                                preferences,
                                lsp_handle,
                            );
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
                                        perform_action(
                                            Action::QuitInsertMode,
                                            state,
                                            preferences,
                                            lsp_handle,
                                        );
                                    }
                                    egui::Key::I => {
                                        if matches!(state.mode, Mode::Normal) {
                                            perform_action(
                                                Action::EnterInsertMode,
                                                state,
                                                preferences,
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
                                                preferences,
                                                lsp_handle,
                                            );
                                            return;
                                        }
                                    }
                                    egui::Key::Comma => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::RemoveIndent,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::CyclePreviousBuffer,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::Period => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::AddIndent,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::CycleNextBuffer,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::Slash => {
                                        if modifiers.ctrl {
                                            perform_action(
                                                Action::CloseCurrentBuffer,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::X => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::SelectAndExtentCurrentLine,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::SelectCurrentLine,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::W => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::ExtendSelectTillEndOfWord,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::SelectTillEndOfWord,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::F => {
                                        if matches!(state.mode, Mode::Normal) {
                                            perform_action(
                                                Action::OpenFile,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                            return;
                                        }
                                    }
                                    egui::Key::S => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::SaveCurrentBuffer,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::FormatCurrentBuffer,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::ArrowDown => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::ExtendCursorDown,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::MoveCursorDown,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::ArrowUp => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::ExtendCursorUp,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::MoveCursorUp,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::ArrowLeft => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::ExtendCursorLeft,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::MoveCursorLeft,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::ArrowRight => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::ExtendCursorRight,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::MoveCursorRight,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::Home => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::ExtendCursorLineStart,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::MoveCursorLineStart,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::End => {
                                        if modifiers.shift {
                                            perform_action(
                                                Action::ExtendCursorLineEnd,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::MoveCursorLineEnd,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::G => {
                                        if !modifiers.shift {
                                            perform_action(
                                                Action::GoToBufferStart,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::GoToBufferEnd,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::Semicolon => {
                                        perform_action(
                                            Action::Unselect,
                                            state,
                                            preferences,
                                            lsp_handle,
                                        );
                                    }
                                    egui::Key::J => {
                                        if matches!(state.mode, Mode::Normal) {
                                            if modifiers.shift {
                                                perform_action(
                                                    Action::ExtendCursorDown,
                                                    state,
                                                    preferences,
                                                    lsp_handle,
                                                );
                                            } else {
                                                perform_action(
                                                    Action::MoveCursorDown,
                                                    state,
                                                    preferences,
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
                                                    preferences,
                                                    lsp_handle,
                                                );
                                            } else {
                                                perform_action(
                                                    Action::MoveCursorUp,
                                                    state,
                                                    preferences,
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
                                                    preferences,
                                                    lsp_handle,
                                                );
                                            } else {
                                                perform_action(
                                                    Action::MoveCursorLeft,
                                                    state,
                                                    preferences,
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
                                                    preferences,
                                                    lsp_handle,
                                                );
                                            } else {
                                                perform_action(
                                                    Action::MoveCursorRight,
                                                    state,
                                                    preferences,
                                                    lsp_handle,
                                                );
                                            }
                                        }
                                    }
                                    egui::Key::Z => {
                                        if !modifiers.shift {
                                            perform_action(
                                                Action::LSPHover,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::LSPCompletion,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        }
                                    }
                                    egui::Key::Backspace => {
                                        perform_action(
                                            Action::DeletePreviousCharacter,
                                            state,
                                            preferences,
                                            lsp_handle,
                                        );
                                    }
                                    egui::Key::Delete => {
                                        perform_action(
                                            Action::DeleteNextCharacter,
                                            state,
                                            preferences,
                                            lsp_handle,
                                        );
                                    }
                                    egui::Key::D => {
                                        perform_action(
                                            Action::DeleteSelection,
                                            state,
                                            preferences,
                                            lsp_handle,
                                        );
                                    }
                                    egui::Key::Enter => {
                                        perform_action(
                                            Action::InsertNewLineAtCursor,
                                            state,
                                            preferences,
                                            lsp_handle,
                                        );
                                    }
                                    egui::Key::Tab => {
                                        perform_action(
                                            Action::AddTab,
                                            state,
                                            preferences,
                                            lsp_handle,
                                        );
                                    }
                                    egui::Key::U => {
                                        if !modifiers.shift {
                                            perform_action(
                                                Action::Undo,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
                                        } else {
                                            perform_action(
                                                Action::Redo,
                                                state,
                                                preferences,
                                                lsp_handle,
                                            );
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
                            state.modal_input.push_str(text);
                            state.modal_options_filtered = state
                                .modal_options
                                .iter()
                                .filter(|entry| entry.path.starts_with(&state.modal_input))
                                .cloned()
                                .collect();
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
                                        if !state.modal_options_filtered.is_empty() {
                                            if state.modal_selection_idx.is_none() {
                                                state.modal_selection_idx = Some(0);
                                            } else {
                                                state.modal_selection_idx =
                                                    Some(state.modal_selection_idx.unwrap() + 1);
                                                if state.modal_selection_idx.unwrap()
                                                    >= state.modal_options_filtered.len()
                                                {
                                                    state.modal_selection_idx = Some(0);
                                                }
                                            }

                                            state.modal_input = state.modal_options_filtered
                                                [state.modal_selection_idx.unwrap()]
                                            .path
                                            .clone();
                                        } else {
                                            state.modal_selection_idx = None;
                                        }
                                    }
                                    egui::Key::Backspace => {
                                        state.modal_input.pop();
                                        state.modal_options_filtered = state
                                            .modal_options
                                            .iter()
                                            .filter(|entry| {
                                                entry.path.starts_with(&state.modal_input)
                                            })
                                            .cloned()
                                            .collect();
                                    }
                                    egui::Key::Enter => {
                                        if state.modal_selection_idx.is_some() {
                                            let entry = &state.modal_options_filtered
                                                [state.modal_selection_idx.unwrap()];
                                            if !entry.is_dir {
                                                let path = entry.path.clone();
                                                let initial_text =
                                                    file_io::read_file_content(&path).unwrap();
                                                let buffer = LineBuffer::new(
                                                    initial_text.clone(),
                                                    Some(path.clone()),
                                                );
                                                state.buffer_idx = Some(state.add_buffer(buffer));
                                                state.modal_open = false;
                                                state.modal_options = vec![];
                                                state.modal_options_filtered = vec![];
                                                state.modal_selection_idx = None;
                                                state.modal_input = "".into();

                                                lsp_handle
                                                    .send_notification_sync(
                                                        "textDocument/didOpen".to_string(),
                                                        Some(
                                                            LSPClientHandle::did_open_text_document(
                                                                path.clone(),
                                                                initial_text,
                                                            ),
                                                        ),
                                                    )
                                                    .unwrap();
                                            } else {
                                                state.modal_input = entry.path.clone();

                                                if modifiers.shift {
                                                    state.workspace_folder = entry.path.clone();
                                                    lsp_handle.init_lsp_sync(
                                                        state.workspace_folder.clone(),
                                                    );
                                                }

                                                #[cfg(target_os = "windows")]
                                                {
                                                    state.modal_input.push('\\');
                                                }

                                                #[cfg(any(
                                                    target_os = "linux",
                                                    target_os = "macos"
                                                ))]
                                                {
                                                    state.modal_input.push('/');
                                                }

                                                state.modal_options =
                                                    file_io::get_directory_entries(&entry.path)
                                                        .unwrap();
                                                state.modal_options_filtered =
                                                    state.modal_options.clone();
                                                state.modal_selection_idx = None;
                                            }
                                        }
                                    }
                                    egui::Key::Escape => {
                                        state.modal_open = false;
                                        state.modal_options = vec![];
                                        state.modal_options_filtered = vec![];
                                        state.modal_selection_idx = None;
                                        state.modal_input = "".into();
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
