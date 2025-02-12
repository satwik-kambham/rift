use std::path;

use copypasta::ClipboardProvider;

use crate::{
    buffer::{
        instance::{Cursor, Selection},
        line_buffer::LineBuffer,
    },
    io::file_io,
    lsp::client::LSPClientHandle,
    state::{EditorState, Mode},
};

#[derive(Debug)]
pub enum Action {
    InsertTextAtCursor(String),
    InsertText(String, Cursor),
    DeleteText(Selection),
    InsertNewLineAtCursor,
    EnterInsertMode,
    QuitInsertMode,
    AddNewLineBelowAndEnterInsertMode,
    InsertAfterSelection,
    AddIndent,
    RemoveIndent,
    CycleNextBuffer,
    CyclePreviousBuffer,
    CloseCurrentBuffer,
    SaveCurrentBuffer,
    SelectCurrentLine,
    SelectAndExtentCurrentLine,
    SelectTillEndOfWord,
    ExtendSelectTillEndOfWord,
    SelectTillStartOfWord,
    ExtendSelectTillStartOfWord,
    OpenFile,
    FormatCurrentBuffer,
    MoveCursorDown,
    MoveCursorUp,
    MoveCursorLeft,
    MoveCursorRight,
    ExtendCursorDown,
    ExtendCursorUp,
    ExtendCursorLeft,
    ExtendCursorRight,
    MoveCursorLineStart,
    MoveCursorLineEnd,
    ExtendCursorLineStart,
    ExtendCursorLineEnd,
    GoToBufferStart,
    GoToBufferEnd,
    Unselect,
    LSPHover,
    LSPCompletion,
    DeletePreviousCharacter,
    DeleteNextCharacter,
    DeleteSelection,
    AddTab,
    Undo,
    Redo,
    CopyToRegister,
    CopyToClipboard,
    CutToRegister,
    CutToClipboard,
    PasteFromRegister,
    PasteFromClipboard,
}

pub fn perform_action(
    action: Action,
    state: &mut EditorState,
    lsp_handle: &mut Option<&mut LSPClientHandle>,
) {
    match action {
        Action::InsertTextAtCursor(text) => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            let cursor = buffer.insert_text(&text, &instance.cursor, lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
        }
        Action::InsertText(text, cursor) => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            let _cursor = buffer.insert_text(&text, &cursor, lsp_handle, true);
            instance.cursor = Cursor { row: 0, column: 0 };
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::DeleteText(selection) => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            let (_text, _cursor) = buffer.remove_text(&selection, lsp_handle, true);
            instance.cursor = Cursor { row: 0, column: 0 };
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::InsertNewLineAtCursor => {
            if matches!(state.mode, Mode::Insert) {
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                instance.cursor = instance.selection.cursor;
                let indent_size = buffer.get_indentation_level(instance.cursor.row);
                let cursor = buffer.insert_text("\n", &instance.cursor, lsp_handle, true);
                instance.cursor = cursor;
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                instance.selection =
                    buffer.add_indentation(&instance.selection, indent_size, lsp_handle);
                instance.cursor = instance.selection.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::EnterInsertMode => {
            if matches!(state.mode, Mode::Normal) {
                state.mode = Mode::Insert;
            }
        }
        Action::QuitInsertMode => {
            state.mode = Mode::Normal;
        }
        Action::AddNewLineBelowAndEnterInsertMode => {
            if matches!(state.mode, Mode::Normal) {
                state.mode = Mode::Insert;
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                instance.cursor = instance.selection.cursor;
                let indent_size = buffer.get_indentation_level(instance.cursor.row);
                buffer.move_cursor_line_end(&mut instance.cursor);
                let cursor = buffer.insert_text("\n", &instance.cursor, lsp_handle, true);
                instance.cursor = cursor;
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                instance.selection =
                    buffer.add_indentation(&instance.selection, indent_size, lsp_handle);
                instance.cursor = instance.selection.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::InsertAfterSelection => {}
        Action::AddIndent => {
            if matches!(state.mode, Mode::Normal) {
                let tab_width = state.preferences.tab_width;
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                instance.selection =
                    buffer.add_indentation(&instance.selection, tab_width, lsp_handle);
                instance.cursor = instance.selection.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::RemoveIndent => {
            if matches!(state.mode, Mode::Normal) {
                let tab_width = state.preferences.tab_width;
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                instance.selection =
                    buffer.remove_indentation(&instance.selection, tab_width, lsp_handle);
                instance.cursor = instance.selection.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::CycleNextBuffer => {
            if matches!(state.mode, Mode::Normal) {
                state.cycle_buffer(false);
            }
        }
        Action::CyclePreviousBuffer => {
            if matches!(state.mode, Mode::Normal) {
                state.cycle_buffer(true);
            }
        }
        Action::CloseCurrentBuffer => {
            if state.buffer_idx.is_some() {
                state.remove_buffer(state.buffer_idx.unwrap());
            }
        }
        Action::SaveCurrentBuffer => {
            if matches!(state.mode, Mode::Normal) {
                let line_ending = state.preferences.line_ending.clone();
                let (buffer, _instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                buffer.modified = false;
                file_io::override_file_content(
                    &buffer.file_path.clone().unwrap(),
                    buffer.get_content(line_ending.to_string()),
                )
                .unwrap();
            }
        }
        Action::SelectCurrentLine => {
            if matches!(state.mode, Mode::Normal) {
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                instance.selection.mark = instance.selection.cursor;
                instance.selection = buffer.select_line(&instance.selection);
                instance.cursor = instance.selection.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::SelectAndExtentCurrentLine => {
            if matches!(state.mode, Mode::Normal) {
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                instance.selection = buffer.select_line(&instance.selection);
                instance.cursor = instance.selection.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::SelectTillEndOfWord => {
            if matches!(state.mode, Mode::Normal) {
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                instance.selection.mark = instance.selection.cursor;
                instance.selection = buffer.select_word(&instance.selection);
                instance.cursor = instance.selection.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::ExtendSelectTillEndOfWord => {
            if matches!(state.mode, Mode::Normal) {
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                instance.selection = buffer.select_word(&instance.selection);
                instance.cursor = instance.selection.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::SelectTillStartOfWord => {}
        Action::ExtendSelectTillStartOfWord => {}
        Action::OpenFile => {
            if matches!(state.mode, Mode::Normal) {
                state.modal.open();
                state.modal.options = file_io::get_directory_entries(&state.workspace_folder)
                    .unwrap()
                    .iter()
                    .map(|entry| entry.name.clone())
                    .collect();
                state.modal.input = state.workspace_folder.clone();
                state
                    .modal
                    .set_modal_on_input(|input, state, _lsp_handles| {
                        file_io::get_directory_entries(&state.workspace_folder)
                            .unwrap()
                            .iter()
                            .filter(|entry| entry.path.starts_with(input))
                            .map(|entry| entry.name.clone())
                            .collect()
                    });
                state.modal.set_modal_on_select(
                    |input, selection, alt_select, state, lsp_handles| {
                        let mut path = path::PathBuf::from(input);
                        path.push(selection);
                        let path_str = path.to_str().unwrap().to_owned();
                        if path.is_dir() {
                            if alt_select {
                                state.workspace_folder = path_str.clone();
                            }

                            // state.modal.input = path_str.clone();
                            // #[cfg(target_os = "windows")]
                            // {
                            //     state.modal.input.push('\\');
                            // }

                            // #[cfg(any(target_os = "linux", target_os = "macos"))]
                            // {
                            //     state.modal.input.push('/');
                            // }

                            // state.modal_options =
                            //     file_io::get_directory_entries(&entry.path).unwrap();
                        } else {
                            let initial_text = file_io::read_file_content(&path_str).unwrap();
                            let buffer =
                                LineBuffer::new(initial_text.clone(), Some(path_str.clone()));

                            if let std::collections::hash_map::Entry::Vacant(e) =
                                lsp_handles.entry(buffer.language)
                            {
                                if let Some(mut lsp_handle) = state.spawn_lsp(buffer.language) {
                                    lsp_handle.init_lsp_sync(state.workspace_folder.clone());
                                    e.insert(lsp_handle);
                                }
                            }

                            if let Some(lsp_handle) = lsp_handles.get(&buffer.language) {
                                lsp_handle
                                    .send_notification_sync(
                                        "textDocument/didOpen".to_string(),
                                        Some(LSPClientHandle::did_open_text_document(
                                            path_str.clone(),
                                            initial_text,
                                        )),
                                    )
                                    .unwrap();
                            }

                            state.buffer_idx = Some(state.add_buffer(buffer));
                            state.modal.close();
                        }
                    },
                );
            }
        }
        Action::FormatCurrentBuffer => {
            if matches!(state.mode, Mode::Normal) {
                let (buffer, _instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                if let Some(lsp_handle) = lsp_handle {
                    lsp_handle
                        .send_request_sync(
                            "textDocument/formatting".to_string(),
                            Some(LSPClientHandle::formatting_request(
                                buffer.file_path.clone().unwrap(),
                                state.preferences.tab_width,
                            )),
                        )
                        .unwrap();
                }
            }
        }
        Action::MoveCursorDown => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.move_cursor_down(&mut instance.cursor, instance.column_level);
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
        }
        Action::MoveCursorUp => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.move_cursor_up(&mut instance.cursor, instance.column_level);
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
        }
        Action::MoveCursorLeft => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.move_cursor_left(&mut instance.cursor);
            instance.selection.cursor = instance.cursor;
            instance.column_level = instance.cursor.column;
            instance.selection.mark = instance.cursor;
        }
        Action::MoveCursorRight => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.move_cursor_right(&mut instance.cursor);
            instance.selection.cursor = instance.cursor;
            instance.column_level = instance.cursor.column;
            instance.selection.mark = instance.cursor;
        }
        Action::ExtendCursorDown => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.move_cursor_down(&mut instance.cursor, instance.column_level);
            instance.selection.cursor = instance.cursor;
        }
        Action::ExtendCursorUp => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.move_cursor_up(&mut instance.cursor, instance.column_level);
            instance.selection.cursor = instance.cursor;
        }
        Action::ExtendCursorLeft => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.move_cursor_left(&mut instance.cursor);
            instance.selection.cursor = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::ExtendCursorRight => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.move_cursor_right(&mut instance.cursor);
            instance.selection.cursor = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::MoveCursorLineStart => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.move_cursor_line_start(&mut instance.cursor);
            instance.selection.cursor = instance.cursor;
            instance.column_level = instance.cursor.column;
            instance.selection.mark = instance.cursor;
        }
        Action::MoveCursorLineEnd => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.move_cursor_line_end(&mut instance.cursor);
            instance.selection.cursor = instance.cursor;
            instance.column_level = instance.cursor.column;
            instance.selection.mark = instance.cursor;
        }
        Action::ExtendCursorLineStart => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.move_cursor_line_start(&mut instance.cursor);
            instance.selection.cursor = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::ExtendCursorLineEnd => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.move_cursor_line_end(&mut instance.cursor);
            instance.selection.cursor = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::GoToBufferStart => {
            if matches!(state.mode, Mode::Normal) {
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                buffer.move_cursor_buffer_start(&mut instance.cursor);
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::GoToBufferEnd => {
            if matches!(state.mode, Mode::Normal) {
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                buffer.move_cursor_buffer_end(&mut instance.cursor);
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::Unselect => {
            let (_buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::LSPHover => {
            if matches!(state.mode, Mode::Normal) {
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                if let Some(lsp_handle) = lsp_handle {
                    lsp_handle
                        .send_request_sync(
                            "textDocument/hover".to_string(),
                            Some(LSPClientHandle::hover_request(
                                buffer.file_path.clone().unwrap(),
                                instance.cursor,
                            )),
                        )
                        .unwrap();
                }
            }
        }
        Action::LSPCompletion => {
            if matches!(state.mode, Mode::Normal) {
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                if let Some(lsp_handle) = lsp_handle {
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
        Action::DeletePreviousCharacter => {
            if matches!(state.mode, Mode::Insert) {
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                buffer.move_cursor_left(&mut instance.selection.mark);

                let (_text, cursor) = buffer.remove_text(&instance.selection, lsp_handle, true);
                instance.cursor = cursor;
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::DeleteNextCharacter => {
            if matches!(state.mode, Mode::Insert) {
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                buffer.move_cursor_right(&mut instance.selection.cursor);

                let (_text, cursor) = buffer.remove_text(&instance.selection, lsp_handle, true);
                instance.cursor = cursor;
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::DeleteSelection => {
            if matches!(state.mode, Mode::Normal) {
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                let (_text, cursor) = buffer.remove_text(&instance.selection, lsp_handle, true);
                instance.cursor = cursor;
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::Undo => {
            if matches!(state.mode, Mode::Normal) {
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                if let Some(cursor) = buffer.undo(lsp_handle) {
                    instance.cursor = cursor;
                    instance.selection.cursor = instance.cursor;
                    instance.selection.mark = instance.cursor;
                    instance.column_level = instance.cursor.column;
                }
            }
        }
        Action::Redo => {
            if matches!(state.mode, Mode::Normal) {
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                if let Some(cursor) = buffer.redo(lsp_handle) {
                    instance.cursor = cursor;
                    instance.selection.cursor = instance.cursor;
                    instance.selection.mark = instance.cursor;
                    instance.column_level = instance.cursor.column;
                }
            }
        }
        Action::AddTab => {
            if matches!(state.mode, Mode::Insert) {
                let tab_width = state.preferences.tab_width;
                let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
                let cursor =
                    buffer.insert_text(&" ".repeat(tab_width), &instance.cursor, lsp_handle, true);
                instance.cursor = cursor;
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::CopyToRegister => {}
        Action::CopyToClipboard => {
            let (buffer, instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
            state
                .clipboard_ctx
                .set_contents(buffer.get_selection(&instance.selection))
                .unwrap();
        }
        Action::CutToRegister => {}
        Action::CutToClipboard => {}
        Action::PasteFromRegister => {}
        Action::PasteFromClipboard => {
            let content = state.clipboard_ctx.get_contents().unwrap();
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            let cursor = buffer.insert_text(&content, &instance.cursor, lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
        }
    }
}
