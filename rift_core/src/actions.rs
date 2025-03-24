use std::{collections::HashMap, path, str::FromStr};

use copypasta::ClipboardProvider;
use strum::VariantNames;
use strum_macros::{EnumIter, EnumMessage, EnumString, VariantNames};

use crate::{
    ai,
    buffer::{
        instance::{Cursor, Language, Selection},
        line_buffer::LineBuffer,
    },
    concurrent::cli::{run_command, run_piped_commands, ProgramArgs},
    io::file_io,
    lsp::client::LSPClientHandle,
    state::{EditorState, Mode},
};

#[derive(Debug, Clone, EnumIter, EnumMessage, EnumString, VariantNames)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum Action {
    Quit,
    InsertTextAtCursor(String),
    InsertTextAtCursorAndTriggerCompletion(String),
    InsertSpace,
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
    Select(Selection),
    SelectCurrentLine,
    SelectAndExtendCurrentLine,
    SelectTillEndOfWord,
    ExtendSelectTillEndOfWord,
    SelectTillStartOfWord,
    ExtendSelectTillStartOfWord,
    CreateBufferFromFile(String),
    OpenFile,
    SwitchBuffer,
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
    LSPSignatureHelp,
    GoToDefinition,
    GoToReferences,
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
    FuzzyFindFile(bool),
    SearchWorkspace,
    WorkspaceDiagnostics,
    LocationModal(Vec<(String, Selection)>),
    OpenCommandDispatcher,
    FIMCompletion,
    KeybindHelp,
}

pub fn perform_action(
    action: Action,
    state: &mut EditorState,
    lsp_handles: &mut HashMap<Language, LSPClientHandle>,
) {
    match action {
        Action::Quit => {
            state.quit = true;
        }
        Action::InsertTextAtCursor(text) => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            let cursor = buffer.insert_text(&text, &instance.cursor, lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
        }
        Action::InsertTextAtCursorAndTriggerCompletion(text) => {
            perform_action(Action::InsertTextAtCursor(text), state, lsp_handles);
            if state.preferences.trigger_completion_on_type {
                perform_action(Action::LSPCompletion, state, lsp_handles);
                perform_action(Action::LSPSignatureHelp, state, lsp_handles);
            }
        }
        Action::InsertSpace => {
            perform_action(
                Action::InsertTextAtCursor(" ".to_string()),
                state,
                lsp_handles,
            );
        }
        Action::InsertText(text, cursor) => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            let cursor = buffer.insert_text(&text, &cursor, lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::DeleteText(selection) => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            let (_text, cursor) = buffer.remove_text(&selection, lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::InsertNewLineAtCursor => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
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
        Action::EnterInsertMode => {
            if matches!(state.mode, Mode::Normal) {
                state.mode = Mode::Insert;
            }
        }
        Action::QuitInsertMode => {
            state.mode = Mode::Normal;
            state.signature_information.content = String::new();
        }
        Action::AddNewLineBelowAndEnterInsertMode => {
            if matches!(state.mode, Mode::Normal) {
                let lsp_handle = if state.buffer_idx.is_some() {
                    let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                    &mut lsp_handles.get_mut(&buffer.language)
                } else {
                    &mut None
                };
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
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
            let tab_width = state.preferences.tab_width;
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            instance.selection = buffer.add_indentation(&instance.selection, tab_width, lsp_handle);
            instance.cursor = instance.selection.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::RemoveIndent => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
            let tab_width = state.preferences.tab_width;
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            instance.selection =
                buffer.remove_indentation(&instance.selection, tab_width, lsp_handle);
            instance.cursor = instance.selection.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::CycleNextBuffer => {
            state.cycle_buffer(false);
        }
        Action::CyclePreviousBuffer => {
            state.cycle_buffer(true);
        }
        Action::CloseCurrentBuffer => {
            if state.buffer_idx.is_some() {
                state.remove_buffer(state.buffer_idx.unwrap());
            }
        }
        Action::SaveCurrentBuffer => {
            let line_ending = state.preferences.line_ending.clone();
            let (buffer, _instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.modified = false;
            file_io::override_file_content(
                &buffer.file_path.clone().unwrap(),
                buffer.get_content(line_ending.to_string()),
            )
            .unwrap();

            if let Some(lsp_handle) = lsp_handles.get(&buffer.language) {
                lsp_handle
                    .send_notification_sync(
                        "textDocument/didSave".to_string(),
                        Some(LSPClientHandle::did_save_text_document(
                            buffer.file_path.clone().unwrap(),
                        )),
                    )
                    .unwrap();
            }
        }
        Action::Select(selection) => {
            let (_buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            instance.selection = selection;
            instance.cursor = instance.selection.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::SelectCurrentLine => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            instance.selection.mark = instance.selection.cursor;
            instance.selection = buffer.select_line(&instance.selection);
            instance.cursor = instance.selection.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::SelectAndExtendCurrentLine => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            instance.selection = buffer.select_line(&instance.selection);
            instance.cursor = instance.selection.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::SelectTillEndOfWord => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            instance.selection.mark = instance.selection.cursor;
            instance.selection = buffer.select_word(&instance.selection);
            instance.cursor = instance.selection.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::ExtendSelectTillEndOfWord => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            instance.selection = buffer.select_word(&instance.selection);
            instance.cursor = instance.selection.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::SelectTillStartOfWord => {}
        Action::ExtendSelectTillStartOfWord => {}
        Action::CreateBufferFromFile(path) => {
            if let Some(idx) = state.buffers.iter().find_map(|(idx, buffer)| {
                if buffer.file_path.clone().unwrap_or_default() == path {
                    Some(idx)
                } else {
                    None
                }
            }) {
                state.buffer_idx = Some(*idx);
            } else {
                let initial_text = file_io::read_file_content(&path).unwrap();
                let buffer = LineBuffer::new(initial_text.clone(), Some(path.clone()));

                if let std::collections::hash_map::Entry::Vacant(e) =
                    lsp_handles.entry(buffer.language)
                {
                    if let Some(mut lsp_handle) = state.spawn_lsp(buffer.language) {
                        lsp_handle.init_lsp_sync(state.workspace_folder.clone());
                        e.insert(lsp_handle);
                    }
                }

                if let Some(lsp_handle) = lsp_handles.get(&buffer.language) {
                    let language_id = match buffer.language {
                        Language::Python => "python",
                        Language::Rust => "rust",
                        Language::Markdown => "markdown",
                        _ => "",
                    };

                    if lsp_handle.initialize_capabilities["textDocumentSync"].is_number()
                        || lsp_handle.initialize_capabilities["textDocumentSync"]["openClose"]
                            .as_bool()
                            .unwrap_or(false)
                    {
                        lsp_handle
                            .send_notification_sync(
                                "textDocument/didOpen".to_string(),
                                Some(LSPClientHandle::did_open_text_document(
                                    path.clone(),
                                    language_id.to_string(),
                                    initial_text,
                                )),
                            )
                            .unwrap();
                    }
                }

                state.buffer_idx = Some(state.add_buffer(buffer));
            };
        }
        Action::OpenFile => {
            state.modal.open();
            state.current_folder = state.workspace_folder.clone();
            state.modal.options = file_io::get_directory_entries(&state.workspace_folder)
                .unwrap()
                .iter()
                .map(|entry| (entry.name.clone(), entry.path.clone()))
                .collect();
            state.modal.input = state.workspace_folder.clone();
            state
                .modal
                .set_modal_on_input(|input, state, _lsp_handles| {
                    state.modal.options = file_io::get_directory_entries(&state.current_folder)
                        .unwrap()
                        .iter()
                        .filter(|entry| entry.path.starts_with(input))
                        .map(|entry| (entry.name.clone(), entry.path.clone()))
                        .collect();
                });
            state
                .modal
                .set_modal_on_select(|_input, selection, alt_select, state, lsp_handles| {
                    let path = path::PathBuf::from(selection.1.clone());
                    let path_str = path.to_str().unwrap().to_owned();
                    if path.is_dir() {
                        state.current_folder = path_str.clone();
                        if alt_select {
                            state.workspace_folder = path_str.clone();
                        }

                        state.modal.input = path_str.clone();
                        #[cfg(target_os = "windows")]
                        {
                            state.modal.input.push('\\');
                        }

                        #[cfg(any(target_os = "linux", target_os = "macos"))]
                        {
                            state.modal.input.push('/');
                        }

                        state.modal.options = file_io::get_directory_entries(&path_str)
                            .unwrap()
                            .iter()
                            .filter(|entry| entry.path.starts_with(&path_str))
                            .map(|entry| (entry.name.clone(), entry.path.clone()))
                            .collect();
                        state.modal.selection = None;
                    } else {
                        perform_action(Action::CreateBufferFromFile(path_str), state, lsp_handles);
                        state.modal.close();
                    }
                });
        }
        Action::SwitchBuffer => {
            state.modal.open();
            state.modal.options = state
                .buffers
                .iter()
                .map(|(idx, buffer)| (buffer.file_path.clone().unwrap(), idx.to_string()))
                .collect();
            state
                .modal
                .set_modal_on_input(|input, state, _lsp_handles| {
                    state.modal.options = state
                        .buffers
                        .iter()
                        .filter(|(_idx, buffer)| buffer.file_path.clone().unwrap().contains(input))
                        .map(|(idx, buffer)| (buffer.file_path.clone().unwrap(), idx.to_string()))
                        .collect();
                });
            state.modal.set_modal_on_select(
                |_input, selection, _alt_select, state, _lsp_handles| {
                    state.buffer_idx = Some(selection.1.parse().unwrap());
                    state.modal.close();
                },
            );
        }
        Action::FormatCurrentBuffer => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
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
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.move_cursor_buffer_start(&mut instance.cursor);
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::GoToBufferEnd => {
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.move_cursor_buffer_end(&mut instance.cursor);
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::Unselect => {
            let (_buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::LSPHover => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
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
        Action::LSPCompletion => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
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
        Action::LSPSignatureHelp => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            if let Some(lsp_handle) = lsp_handle {
                lsp_handle
                    .send_request_sync(
                        "textDocument/signatureHelp".to_string(),
                        Some(LSPClientHandle::signature_help_request(
                            buffer.file_path.clone().unwrap(),
                            instance.cursor,
                        )),
                    )
                    .unwrap();
            }
        }
        Action::GoToDefinition => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            if let Some(lsp_handle) = lsp_handle {
                lsp_handle
                    .send_request_sync(
                        "textDocument/definition".to_string(),
                        Some(LSPClientHandle::go_to_definition_request(
                            buffer.file_path.clone().unwrap(),
                            instance.cursor,
                        )),
                    )
                    .unwrap();
            }
        }
        Action::GoToReferences => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            if let Some(lsp_handle) = lsp_handle {
                lsp_handle
                    .send_request_sync(
                        "textDocument/references".to_string(),
                        Some(LSPClientHandle::go_to_references_request(
                            buffer.file_path.clone().unwrap(),
                            instance.cursor,
                        )),
                    )
                    .unwrap();
            }
        }
        Action::DeletePreviousCharacter => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
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
        Action::DeleteNextCharacter => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
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
        Action::DeleteSelection => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            let (_text, cursor) = buffer.remove_text(&instance.selection, lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::Undo => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            if let Some(cursor) = buffer.undo(lsp_handle) {
                instance.cursor = cursor;
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::Redo => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            if let Some(cursor) = buffer.redo(lsp_handle) {
                instance.cursor = cursor;
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::AddTab => {
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
            let tab_width = state.preferences.tab_width;
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            let cursor =
                buffer.insert_text(&" ".repeat(tab_width), &instance.cursor, lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
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
            let lsp_handle = if state.buffer_idx.is_some() {
                let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
                &mut lsp_handles.get_mut(&buffer.language)
            } else {
                &mut None
            };
            let content = state.clipboard_ctx.get_contents().unwrap();
            let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            let cursor = buffer.insert_text(&content, &instance.cursor, lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
        }
        Action::FuzzyFindFile(_respect_ignore) => {
            run_piped_commands(
                vec![
                    ProgramArgs {
                        program: "fd".into(),
                        args: vec![
                            "--type".to_string(),
                            "f".to_string(),
                            "--strip-cwd-prefix".to_string(),
                            "--full-path".to_string(),
                            state.workspace_folder.clone(),
                        ],
                    },
                    ProgramArgs {
                        program: "fzf".into(),
                        args: vec!["-f".to_string(), "".to_string()],
                    },
                ],
                |result, state, _lsp_handle| {
                    let results: Vec<&str> = result.trim().lines().collect();
                    state.modal.open();
                    state.modal.options = results
                        .iter()
                        .map(|path| (path.to_string(), path.to_string()))
                        .collect();
                    state
                        .modal
                        .set_modal_on_input(|input, state, _lsp_handles| {
                            run_piped_commands(
                                vec![
                                    ProgramArgs {
                                        program: "fd".into(),
                                        args: vec![
                                            "--type".to_string(),
                                            "f".to_string(),
                                            "--strip-cwd-prefix".to_string(),
                                            "--full-path".to_string(),
                                            state.workspace_folder.clone(),
                                        ],
                                    },
                                    ProgramArgs {
                                        program: "fzf".into(),
                                        args: vec!["-f".to_string(), input.to_string()],
                                    },
                                ],
                                |result, state, _lsp_handle| {
                                    let results: Vec<&str> = result.trim().lines().collect();
                                    state.modal.options = results
                                        .iter()
                                        .map(|path| (path.to_string(), path.to_string()))
                                        .collect();
                                },
                                &state.rt,
                                state.async_handle.sender.clone(),
                            );
                        });
                    state.modal.set_modal_on_select(
                        |_input, selection, _alt_select, state, lsp_handles| {
                            let mut path = path::PathBuf::from(selection.0.clone());
                            if path.is_relative() {
                                path = std::path::absolute(path).unwrap();
                            }
                            perform_action(
                                Action::CreateBufferFromFile(path.to_str().unwrap().to_string()),
                                state,
                                lsp_handles,
                            );
                            state.modal.close();
                        },
                    );
                },
                &state.rt,
                state.async_handle.sender.clone(),
            );
        }
        Action::SearchWorkspace => {
            state.modal.open();
            state
                .modal
                .set_modal_on_input(|input, state, _lsp_handles| {
                    if !input.trim().is_empty() {
                        run_command(
                            ProgramArgs {
                                program: "rg".into(),
                                args: vec![
                                    "--json".to_string(),
                                    input.clone(),
                                    state.workspace_folder.clone(),
                                ],
                            },
                            |result, state, _lsp_handle| {
                                let results: Vec<&str> = result.trim().lines().collect();
                                let mut matches: Vec<(String, String)> = vec![];
                                for result in results {
                                    let line_match: serde_json::Value =
                                        serde_json::from_str(result).unwrap();
                                    if line_match["type"] == "match" {
                                        let file_path =
                                            line_match["data"]["path"]["text"].as_str().unwrap();
                                        let row =
                                            line_match["data"]["line_number"].as_u64().unwrap() - 1;
                                        let submatches =
                                            line_match["data"]["submatches"].as_array().unwrap();
                                        let line = line_match["data"]["lines"]["text"]
                                            .as_str()
                                            .unwrap()
                                            .to_string();
                                        for submatch in submatches {
                                            let start = submatch["start"].as_u64().unwrap();
                                            let end = submatch["end"].as_u64().unwrap();
                                            let mut line = line.clone();
                                            line.insert(end.try_into().unwrap(), '<');
                                            line.insert(start.try_into().unwrap(), '>');
                                            let line = line.trim();
                                            matches.push((
                                                format!("{} {}: {}", file_path, row, line),
                                                serde_json::to_string(&serde_json::json!({
                                                    "file_path": file_path,
                                                    "row": row,
                                                    "start": start,
                                                    "end": end,
                                                }))
                                                .unwrap(),
                                            ));
                                        }
                                    }
                                }
                                state.modal.options = matches;
                            },
                            &state.rt,
                            state.async_handle.sender.clone(),
                        );
                    }
                });
            state.modal.set_modal_on_select(
                |_input, selection, _alt_select, state, lsp_handles| {
                    let pattern_match: serde_json::Value =
                        serde_json::from_str(&selection.1).unwrap();
                    let file_path = pattern_match["file_path"].as_str().unwrap();
                    let row = pattern_match["row"].as_u64().unwrap();
                    let start = pattern_match["start"].as_u64().unwrap();
                    let end = pattern_match["end"].as_u64().unwrap();
                    let mut path = path::PathBuf::from(file_path);
                    if path.is_relative() {
                        path = std::path::absolute(path).unwrap();
                    }
                    perform_action(
                        Action::CreateBufferFromFile(path.to_str().unwrap().to_string()),
                        state,
                        lsp_handles,
                    );
                    perform_action(
                        Action::Select(Selection {
                            cursor: Cursor {
                                row: row.try_into().unwrap(),
                                column: end.try_into().unwrap(),
                            },
                            mark: Cursor {
                                row: row.try_into().unwrap(),
                                column: start.try_into().unwrap(),
                            },
                        }),
                        state,
                        lsp_handles,
                    );
                    state.modal.close();
                },
            );
        }
        Action::WorkspaceDiagnostics => {
            state.modal.open();
            let mut workspace_diagnostics: Vec<(String, String)> = vec![];
            for (file_path, diagnostics) in &state.diagnostics {
                for (idx, diagnostic) in diagnostics.diagnostics.iter().enumerate() {
                    workspace_diagnostics.push((
                        diagnostic.message.clone(),
                        serde_json::to_string(&serde_json::json!({
                            "file_path": file_path,
                            "idx": idx,
                        }))
                        .unwrap(),
                    ));
                }
            }
            state.modal.options = workspace_diagnostics;
            state
                .modal
                .set_modal_on_input(|input, state, _lsp_handles| {
                    let mut workspace_diagnostics: Vec<(String, String)> = vec![];
                    for (file_path, diagnostics) in &state.diagnostics {
                        for (idx, diagnostic) in diagnostics.diagnostics.iter().enumerate() {
                            if diagnostic.message.contains(input) {
                                workspace_diagnostics.push((
                                    diagnostic.message.clone(),
                                    serde_json::to_string(&serde_json::json!({
                                        "file_path": file_path,
                                        "idx": idx,
                                    }))
                                    .unwrap(),
                                ));
                            }
                        }
                    }
                    state.modal.options = workspace_diagnostics;
                });
            state.modal.set_modal_on_select(
                |_input, selection, _alt_select, state, lsp_handles| {
                    let selection: serde_json::Value = serde_json::from_str(&selection.1).unwrap();
                    let file_path = selection["file_path"].as_str().unwrap().to_string();

                    #[cfg(target_os = "windows")]
                    let file_path = file_path.to_lowercase();

                    let diagnostic_idx: usize =
                        selection["idx"].as_u64().unwrap().try_into().unwrap();

                    perform_action(
                        Action::CreateBufferFromFile(file_path.clone()),
                        state,
                        lsp_handles,
                    );
                    let diagnostics = state.diagnostics.get(&file_path).unwrap();
                    let diagnostic = &diagnostics.diagnostics[diagnostic_idx];
                    perform_action(
                        Action::Select(Selection {
                            cursor: Cursor {
                                row: diagnostic.range.cursor.row,
                                column: diagnostic.range.cursor.column,
                            },
                            mark: Cursor {
                                row: diagnostic.range.mark.row,
                                column: diagnostic.range.mark.column,
                            },
                        }),
                        state,
                        lsp_handles,
                    );
                    state.modal.close();
                },
            );
        }
        Action::LocationModal(locations) => {
            state.modal.open();
            let mut location_input: Vec<(String, String)> = vec![];
            for (file_path, range) in locations {
                location_input.push((
                    format!("{}:{}-{}", file_path, range.cursor.row, range.cursor.column),
                    serde_json::to_string(&serde_json::json!({
                        "file_path": file_path,
                        "range": range,
                    }))
                    .unwrap(),
                ));
            }
            state.modal.options = location_input;
            state.modal.set_modal_on_select(
                |_input, selection, _alt_select, state, lsp_handles| {
                    let selection: serde_json::Value = serde_json::from_str(&selection.1).unwrap();
                    let file_path = selection["file_path"].as_str().unwrap().to_string();
                    let range: Selection =
                        serde_json::from_value(selection["range"].clone()).unwrap();

                    perform_action(
                        Action::CreateBufferFromFile(file_path.clone()),
                        state,
                        lsp_handles,
                    );
                    perform_action(Action::Select(range), state, lsp_handles);
                    state.modal.close();
                },
            );
        }
        Action::OpenCommandDispatcher => {
            if matches!(state.mode, Mode::Normal) {
                state.modal.open();
                let mut actions: Vec<(String, String)> = vec![];
                for action in Action::VARIANTS {
                    actions.push((action.to_string(), action.to_string()));
                }
                state.modal.options = actions;
                state
                    .modal
                    .set_modal_on_input(|input, state, _lsp_handles| {
                        let mut actions: Vec<(String, String)> = vec![];
                        for action in Action::VARIANTS {
                            if action.contains(input) {
                                actions.push((action.to_string(), action.to_string()));
                            }
                        }
                        state.modal.options = actions;
                    });
                state.modal.set_modal_on_select(
                    |_input, selection, _alt_select, state, lsp_handles| {
                        state.modal.close();
                        let action = Action::from_str(&selection.1).unwrap();
                        perform_action(action, state, lsp_handles);
                    },
                );
            }
        }
        Action::FIMCompletion => {
            ai::ollama_fim(state);
        }
        Action::KeybindHelp => {
            let help_content = state
                .keybind_handler
                .keybinds
                .iter()
                .map(|keybind| keybind.definition.clone())
                .collect::<Vec<_>>()
                .join("\n");
            state.info_modal.open(help_content);
        }
    }
}
