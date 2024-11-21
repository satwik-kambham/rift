use egui::Ui;
use rift_core::{
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
                            if matches!(state.mode, Mode::Insert) {
                                let (buffer, instance) =
                                    state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                let cursor = buffer.insert_text(text, &instance.cursor, lsp_handle);
                                instance.cursor = cursor;
                                instance.selection.cursor = instance.cursor;
                                instance.selection.mark = instance.cursor;
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
                                        state.mode = Mode::Normal;
                                    }
                                    egui::Key::I => {
                                        if matches!(state.mode, Mode::Normal) {
                                            state.mode = Mode::Insert;
                                            return;
                                        }
                                    }
                                    egui::Key::O => {
                                        if matches!(state.mode, Mode::Normal) {
                                            state.mode = Mode::Insert;
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            instance.cursor = instance.selection.cursor;
                                            let indent_size =
                                                buffer.get_indentation_level(instance.cursor.row);
                                            buffer.move_cursor_line_end(&mut instance.cursor);
                                            let cursor = buffer.insert_text(
                                                "\n",
                                                &instance.cursor,
                                                lsp_handle,
                                            );
                                            instance.cursor = cursor;
                                            instance.selection.cursor = instance.cursor;
                                            instance.selection.mark = instance.cursor;
                                            instance.selection = buffer.add_indentation(
                                                &instance.selection,
                                                indent_size,
                                                lsp_handle,
                                            );
                                            instance.cursor = instance.selection.cursor;
                                            instance.column_level = instance.cursor.column;
                                            return;
                                        }
                                    }
                                    egui::Key::Comma => {
                                        if matches!(state.mode, Mode::Normal) {
                                            if modifiers.shift {
                                                let (buffer, instance) = state
                                                    .get_buffer_by_id_mut(
                                                        state.buffer_idx.unwrap(),
                                                    );
                                                instance.selection = buffer.remove_indentation(
                                                    &instance.selection,
                                                    preferences.tab_width,
                                                    lsp_handle,
                                                );
                                                instance.cursor = instance.selection.cursor;
                                                instance.column_level = instance.cursor.column;
                                            } else if modifiers.ctrl {
                                                state.cycle_buffer(true);
                                            }
                                        }
                                    }
                                    egui::Key::Period => {
                                        if matches!(state.mode, Mode::Normal) {
                                            if modifiers.shift {
                                                let (buffer, instance) = state
                                                    .get_buffer_by_id_mut(
                                                        state.buffer_idx.unwrap(),
                                                    );
                                                instance.selection = buffer.add_indentation(
                                                    &instance.selection,
                                                    preferences.tab_width,
                                                    lsp_handle,
                                                );
                                                instance.cursor = instance.selection.cursor;
                                                instance.column_level = instance.cursor.column;
                                            } else if modifiers.ctrl {
                                                state.cycle_buffer(false);
                                            }
                                        }
                                    }
                                    egui::Key::Slash => {
                                        if modifiers.ctrl && state.buffer_idx.is_some() {
                                            state.remove_buffer(state.buffer_idx.unwrap());
                                        }
                                    }
                                    egui::Key::X => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            if !modifiers.shift {
                                                instance.selection.mark = instance.selection.cursor;
                                            }
                                            instance.selection =
                                                buffer.select_line(&instance.selection);
                                            instance.cursor = instance.selection.cursor;
                                            instance.column_level = instance.cursor.column;
                                        }
                                    }
                                    egui::Key::W => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            if !modifiers.shift {
                                                instance.selection.mark = instance.selection.cursor;
                                            }
                                            instance.selection =
                                                buffer.select_word(&instance.selection);
                                            instance.cursor = instance.selection.cursor;
                                            instance.column_level = instance.cursor.column;
                                        }
                                    }
                                    egui::Key::F => {
                                        if matches!(state.mode, Mode::Normal) {
                                            state.modal_open = true;
                                            state.modal_options = file_io::get_directory_entries(
                                                &state.workspace_folder,
                                            )
                                            .unwrap();
                                            state.modal_options_filtered =
                                                state.modal_options.clone();
                                            state.modal_selection_idx = None;
                                            state.modal_input = state.workspace_folder.clone();
                                            return;
                                        }
                                    }
                                    egui::Key::S => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, _instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            if modifiers.shift {
                                                buffer.modified = false;
                                                file_io::override_file_content(
                                                    &buffer.file_path.clone().unwrap(),
                                                    buffer.get_content(
                                                        preferences.line_ending.to_string(),
                                                    ),
                                                )
                                                .unwrap();
                                            } else {
                                                lsp_handle
                                                    .send_request_sync(
                                                        "textDocument/formatting".to_string(),
                                                        Some(LSPClientHandle::formatting_request(
                                                            buffer.file_path.clone().unwrap(),
                                                            preferences.tab_width,
                                                        )),
                                                    )
                                                    .unwrap();
                                            }
                                        }
                                    }
                                    egui::Key::ArrowDown => {
                                        let (buffer, instance) =
                                            state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                        buffer.move_cursor_down(
                                            &mut instance.cursor,
                                            instance.column_level,
                                        );
                                        instance.selection.cursor = instance.cursor;
                                        if !modifiers.shift {
                                            instance.selection.mark = instance.cursor;
                                        }
                                    }
                                    egui::Key::ArrowUp => {
                                        let (buffer, instance) =
                                            state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                        buffer.move_cursor_up(
                                            &mut instance.cursor,
                                            instance.column_level,
                                        );
                                        instance.selection.cursor = instance.cursor;
                                        if !modifiers.shift {
                                            instance.selection.mark = instance.cursor;
                                        }
                                    }
                                    egui::Key::ArrowLeft => {
                                        let (buffer, instance) =
                                            state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                        buffer.move_cursor_left(&mut instance.cursor);
                                        instance.selection.cursor = instance.cursor;
                                        instance.column_level = instance.cursor.column;
                                        if !modifiers.shift {
                                            instance.selection.mark = instance.cursor;
                                        }
                                    }
                                    egui::Key::ArrowRight => {
                                        let (buffer, instance) =
                                            state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                        buffer.move_cursor_right(&mut instance.cursor);
                                        instance.selection.cursor = instance.cursor;
                                        instance.column_level = instance.cursor.column;
                                        if !modifiers.shift {
                                            instance.selection.mark = instance.cursor;
                                        }
                                    }
                                    egui::Key::Home => {
                                        let (buffer, instance) =
                                            state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                        buffer.move_cursor_line_start(&mut instance.cursor);
                                        instance.selection.cursor = instance.cursor;
                                        instance.column_level = instance.cursor.column;
                                        if !modifiers.shift {
                                            instance.selection.mark = instance.cursor;
                                        }
                                    }
                                    egui::Key::End => {
                                        let (buffer, instance) =
                                            state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                        buffer.move_cursor_line_end(&mut instance.cursor);
                                        instance.selection.cursor = instance.cursor;
                                        instance.column_level = instance.cursor.column;
                                        if !modifiers.shift {
                                            instance.selection.mark = instance.cursor;
                                        }
                                    }
                                    egui::Key::G => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            if !modifiers.shift {
                                                buffer
                                                    .move_cursor_buffer_start(&mut instance.cursor);
                                            } else {
                                                buffer.move_cursor_buffer_end(&mut instance.cursor);
                                            }
                                            instance.selection.cursor = instance.cursor;
                                            instance.selection.mark = instance.cursor;
                                            instance.column_level = instance.cursor.column;
                                        }
                                    }
                                    egui::Key::Semicolon => {
                                        let (_buffer, instance) =
                                            state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                        instance.selection.cursor = instance.cursor;
                                        instance.selection.mark = instance.cursor;
                                        instance.column_level = instance.cursor.column;
                                    }
                                    egui::Key::J => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            buffer.move_cursor_down(
                                                &mut instance.cursor,
                                                instance.column_level,
                                            );
                                            instance.selection.cursor = instance.cursor;
                                            if !modifiers.shift {
                                                instance.selection.mark = instance.cursor;
                                            }
                                        }
                                    }
                                    egui::Key::K => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            buffer.move_cursor_up(
                                                &mut instance.cursor,
                                                instance.column_level,
                                            );
                                            instance.selection.cursor = instance.cursor;
                                            if !modifiers.shift {
                                                instance.selection.mark = instance.cursor;
                                            }
                                        }
                                    }
                                    egui::Key::H => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            buffer.move_cursor_left(&mut instance.cursor);
                                            instance.selection.cursor = instance.cursor;
                                            instance.column_level = instance.cursor.column;
                                            if !modifiers.shift {
                                                instance.selection.mark = instance.cursor;
                                            }
                                        }
                                    }
                                    egui::Key::L => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            buffer.move_cursor_right(&mut instance.cursor);
                                            instance.selection.cursor = instance.cursor;
                                            instance.column_level = instance.cursor.column;
                                            if !modifiers.shift {
                                                instance.selection.mark = instance.cursor;
                                            }
                                        }
                                    }
                                    egui::Key::Z => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());

                                            if !modifiers.shift {
                                                lsp_handle
                                                    .send_request_sync(
                                                        "textDocument/hover".to_string(),
                                                        Some(LSPClientHandle::hover_request(
                                                            buffer.file_path.clone().unwrap(),
                                                            instance.cursor,
                                                        )),
                                                    )
                                                    .unwrap();
                                            } else {
                                                lsp_handle
                                                    .send_request_sync(
                                                        "textDocument/completion".to_string(),
                                                        Some(LSPClientHandle::completion_request(
                                                            buffer.file_path.clone().unwrap(),
                                                            instance.cursor,
                                                        )),
                                                    )
                                                    .unwrap();
                                            }
                                        }
                                    }
                                    egui::Key::Backspace => {
                                        if matches!(state.mode, Mode::Insert) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            instance.selection.cursor = instance.cursor;
                                            instance.selection.mark = instance.cursor;
                                            buffer.move_cursor_left(&mut instance.selection.mark);

                                            let (_text, cursor) =
                                                buffer.remove_text(&instance.selection, lsp_handle);
                                            instance.cursor = cursor;
                                            instance.selection.cursor = instance.cursor;
                                            instance.selection.mark = instance.cursor;
                                            instance.column_level = instance.cursor.column;
                                        }
                                    }
                                    egui::Key::Enter => {
                                        if matches!(state.mode, Mode::Insert) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            instance.cursor = instance.selection.cursor;
                                            let indent_size =
                                                buffer.get_indentation_level(instance.cursor.row);
                                            let cursor = buffer.insert_text(
                                                "\n",
                                                &instance.cursor,
                                                lsp_handle,
                                            );
                                            instance.cursor = cursor;
                                            instance.selection.cursor = instance.cursor;
                                            instance.selection.mark = instance.cursor;
                                            instance.selection = buffer.add_indentation(
                                                &instance.selection,
                                                indent_size,
                                                lsp_handle,
                                            );
                                            instance.cursor = instance.selection.cursor;
                                            instance.column_level = instance.cursor.column;
                                        }
                                    }
                                    egui::Key::Tab => {
                                        if matches!(state.mode, Mode::Insert) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            let cursor = buffer.insert_text(
                                                &" ".repeat(preferences.tab_width),
                                                &instance.cursor,
                                                lsp_handle,
                                            );
                                            instance.cursor = cursor;
                                            instance.selection.cursor = instance.cursor;
                                            instance.selection.mark = instance.cursor;
                                            instance.column_level = instance.cursor.column;
                                        }
                                    }
                                    egui::Key::U => {
                                        if matches!(state.mode, Mode::Normal) {
                                            let (buffer, instance) = state
                                                .get_buffer_by_id_mut(state.buffer_idx.unwrap());
                                            if !modifiers.shift {
                                                if let Some(cursor) = buffer.undo() {
                                                    instance.cursor = cursor;
                                                    instance.selection.cursor = instance.cursor;
                                                    instance.selection.mark = instance.cursor;
                                                    instance.column_level = instance.cursor.column;
                                                }
                                            } else if let Some(cursor) = buffer.redo() {
                                                instance.cursor = cursor;
                                                instance.selection.cursor = instance.cursor;
                                                instance.selection.mark = instance.cursor;
                                                instance.column_level = instance.cursor.column;
                                            }
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
