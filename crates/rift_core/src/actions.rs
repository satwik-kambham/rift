use std::{path, str::FromStr};

use copypasta::ClipboardProvider;
use serde_json::json;
use strum::{EnumIter, EnumMessage, EnumString, VariantNames};
use tracing::{error, warn};

use crate::{
    buffer::{
        instance::{Cursor, Selection},
        rope_buffer::RopeBuffer,
    },
    concurrent::cli::{ProgramArgs, run_command},
    io::file_io,
    lsp::{client::LSPClientHandle, types::DiagnosticSeverity},
    preferences::Preferences,
    state::{EditorState, Mode},
};

#[derive(serde::Serialize)]
struct BufferListEntry {
    id: u32,
    display_name: String,
    special: bool,
    modified: bool,
    is_active: bool,
}

#[derive(serde::Serialize)]
struct WorkspaceDiagnosticEntry {
    file_path: String,
    message: String,
    severity: String,
    source: String,
    code: String,
    range: Selection,
}

#[derive(serde::Serialize, Clone)]
pub struct ReferenceEntry {
    pub file_path: String,
    pub range: Selection,
    pub preview: String,
}

fn rsl_string_literal(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

pub fn open_info_modal_in_rsl(state: &mut EditorState, content: &str) {
    let serialized = rsl_string_literal(content);
    perform_action(
        Action::RunSource(format!("infoModalOpen({})", serialized)),
        state,
    );
}

fn diagnostic_severity_label(severity: &DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Hint => "Hint",
        DiagnosticSeverity::Information => "Information",
        DiagnosticSeverity::Warning => "Warning",
        DiagnosticSeverity::Error => "Error",
    }
}

#[derive(Debug, Clone, EnumIter, EnumMessage, EnumString, VariantNames)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum Action {
    Quit,
    SetBufferContent(u32, String),
    InsertBufferInput(String),
    GetWorkspaceDir,
    GetViewportSize,
    GetBufferInput(u32),
    SetBufferInput(u32, String),
    InsertTextAtCursor(String),
    InsertTextAtCursorAndTriggerCompletion(String),
    InsertSpace,
    InsertText(String, Cursor),
    DeleteText(Selection),
    InsertNewLineAtCursor,
    EnterInsertMode,
    QuitInsertMode,
    DeleteSelectionAndEnterInsertMode,
    AddNewLineBelowAndEnterInsertMode,
    InsertAfterSelection,
    AddIndent,
    RemoveIndent,
    ToggleComment,
    SetActiveBuffer(u32),
    GetActiveBuffer,
    CycleNextBuffer,
    CyclePreviousBuffer,
    CloseCurrentBuffer,
    SaveCurrentBuffer,
    RunCurrentBuffer,
    RunSource(String),
    Select(Selection),
    SelectAndExtendCurrentLine,
    SelectTillEndOfWord,
    ExtendSelectTillEndOfWord,
    SelectTillStartOfWord,
    ExtendSelectTillStartOfWord,
    CreateBufferFromFile(String),
    CreateSpecialBuffer(String),
    OpenFile(String),
    ListBuffers,
    GetActions,
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
    GetDefinitions,
    GoToReferences,
    GetReferences,
    DeletePreviousCharacter,
    DeleteNextCharacter,
    DeleteSelection,
    AddTab,
    Undo,
    Redo,
    CopyToRegister,
    CopyToClipboard,
    PasteFromRegister,
    PasteFromClipboard,
    SetSearchQuery(String),
    SetSearchQueryFromSelectionOrPrompt,
    FindNextWithQuery,
    SearchWorkspace,
    GetWorkspaceDiagnostics,
    WorkspaceDiagnostics,
    RunAction(String),
    OpenCommandDispatcher,
    KeybindHelp,
    IncreaseFontSize,
    DecreaseFontSize,
    ScrollDown,
    ScrollUp,
    Log(String),
    SetInitRslComplete,
    RegisterGlobalKeybind(String, String),
    RegisterBufferKeybind(u32, String, String),
    RegisterBufferInputHook(u32, String),
}

pub fn perform_action(action: Action, state: &mut EditorState) -> Option<String> {
    match action {
        Action::Quit => {
            state.quit = true;
        }
        Action::SetBufferContent(buffer_id, content) => {
            let (buffer, _instance) = state.get_buffer_by_id_mut(buffer_id);
            buffer.set_content(content.clone());
        }
        Action::InsertBufferInput(text) => {
            let (buffer, _instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            buffer.input.push_str(&text);
            if let Some(function_id) = &buffer.input_hook {
                perform_action(
                    Action::RunSource(format!("runFunctionById(\"{}\")", function_id)),
                    state,
                );
            }
        }
        Action::GetWorkspaceDir => {
            return Some(state.workspace_folder.clone());
        }
        Action::GetViewportSize => {
            let size = json!({
                "rows": state.viewport_rows(),
                "columns": state.viewport_columns(),
            });
            return Some(size.to_string());
        }
        Action::GetBufferInput(buffer_id) => {
            let (buffer, _instance) = state.get_buffer_by_id_mut(buffer_id);
            return Some(buffer.input.clone());
        }
        Action::SetBufferInput(buffer_id, text) => {
            let (buffer, _instance) = state.get_buffer_by_id_mut(buffer_id);
            buffer.input = text;
            if let Some(function_id) = &buffer.input_hook {
                perform_action(
                    Action::RunSource(format!("runFunctionById(\"{}\")", function_id)),
                    state,
                );
            }
        }
        Action::InsertTextAtCursor(text) => {
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            let cursor = buffer.insert_text(&text, &instance.cursor, &lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;

            // Dismiss LSP Completion and LSP Signature Help
            state.completion_menu.close();
            state.signature_information.content = String::new();
        }
        Action::InsertTextAtCursorAndTriggerCompletion(text) => {
            perform_action(Action::InsertTextAtCursor(text), state);
            if state.preferences.trigger_completion_on_type {
                perform_action(Action::LSPCompletion, state);
                perform_action(Action::LSPSignatureHelp, state);
            }
        }
        Action::InsertSpace => {
            perform_action(Action::InsertTextAtCursor(" ".to_string()), state);
        }
        Action::InsertText(text, cursor) => {
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            let cursor = buffer.insert_text(&text, &cursor, &lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::DeleteText(selection) => {
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            let (_text, cursor) = buffer.remove_text(&selection, &lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::InsertNewLineAtCursor => {
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            instance.cursor = instance.selection.cursor;
            let indent_size = buffer.get_indentation_level(instance.cursor.row);
            let cursor = buffer.insert_text("\n", &instance.cursor, &lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.selection =
                buffer.add_indentation(&instance.selection, indent_size, &lsp_handle);
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
        Action::DeleteSelectionAndEnterInsertMode => {
            if matches!(state.mode, Mode::Normal) {
                perform_action(Action::DeleteSelection, state);
                perform_action(Action::EnterInsertMode, state);
            }
        }
        Action::AddNewLineBelowAndEnterInsertMode => {
            if matches!(state.mode, Mode::Normal) {
                state.mode = Mode::Insert;
                let (buffer, instance, lsp_handle) =
                    state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
                instance.cursor = instance.selection.cursor;
                let indent_size = buffer.get_indentation_level(instance.cursor.row);
                buffer.move_cursor_line_end(&mut instance.cursor);
                let cursor = buffer.insert_text("\n", &instance.cursor, &lsp_handle, true);
                instance.cursor = cursor;
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                instance.selection =
                    buffer.add_indentation(&instance.selection, indent_size, &lsp_handle);
                instance.cursor = instance.selection.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::InsertAfterSelection => {}
        Action::AddIndent => {
            let tab_width = state.preferences.tab_width;
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            instance.selection =
                buffer.add_indentation(&instance.selection, tab_width, &lsp_handle);
            instance.cursor = instance.selection.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::RemoveIndent => {
            let tab_width = state.preferences.tab_width;
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            instance.selection =
                buffer.remove_indentation(&instance.selection, tab_width, &lsp_handle);
            instance.cursor = instance.selection.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::ToggleComment => {
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            let comment_token = Preferences::get_comment_token(buffer.language);
            instance.selection =
                buffer.toggle_comment(&instance.selection, comment_token, &lsp_handle);
            instance.cursor = instance.selection.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::SetActiveBuffer(buffer_id) => {
            state.buffer_idx = Some(buffer_id);
        }
        Action::GetActiveBuffer => {
            return Some(
                state
                    .buffer_idx
                    .map(|id| id.to_string())
                    .unwrap_or_default(),
            );
        }
        Action::CycleNextBuffer => {
            state.cycle_buffer(false, true);
        }
        Action::CyclePreviousBuffer => {
            state.cycle_buffer(true, true);
        }
        Action::CloseCurrentBuffer => {
            if state.buffer_idx.is_some() {
                state.remove_buffer(state.buffer_idx.unwrap());
            }
        }
        Action::SaveCurrentBuffer => {
            let line_ending = state.preferences.line_ending.clone();
            let (buffer, _instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            let file_path = match buffer.file_path() {
                Some(path) => path.clone(),
                None => {
                    warn!("Attempted to save buffer without a file path");
                    return None;
                }
            };
            let content = buffer.get_content(line_ending.to_string());
            if let Err(err) = file_io::override_file_content(&file_path, content) {
                error!(%err, path = %file_path, "Failed to save buffer");
                return None;
            }
            buffer.modified = false;

            if let Some(lsp_handle) = lsp_handle
                && let Err(err) = lsp_handle.lock().unwrap().send_notification_sync(
                    "textDocument/didSave".to_string(),
                    Some(LSPClientHandle::did_save_text_document(file_path)),
                )
            {
                warn!(%err, "Failed to send didSave notification");
            }
        }
        Action::RunCurrentBuffer => {
            let (buffer, _instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            let source = buffer.get_content("\n".to_string());
            perform_action(Action::RunSource(source), state);
        }
        Action::RunSource(source) => {
            state.rsl_sender.blocking_send(source).unwrap();
        }
        Action::Select(selection) => {
            let (_buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            instance.selection = selection;
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
        Action::CreateSpecialBuffer(display_name) => {
            let mut buffer = RopeBuffer::new(String::new(), None, &state.workspace_folder, true);
            if !display_name.is_empty() {
                buffer.display_name = Some(display_name);
            }
            let buffer_id = state.add_buffer(buffer);
            return Some(buffer_id.to_string());
        }
        Action::CreateBufferFromFile(path) => {
            if let Some(idx) = state.buffers.iter().find_map(|(idx, buffer)| {
                if buffer.file_path().cloned().unwrap_or_default() == path {
                    Some(idx)
                } else {
                    None
                }
            }) {
                state.buffer_idx = Some(*idx);
            } else {
                let initial_text = match file_io::read_file_content(&path) {
                    Ok(text) => text,
                    Err(err) => {
                        error!(%err, path = %path, "Failed to open file");
                        return None;
                    }
                };
                let buffer = RopeBuffer::new(
                    initial_text.clone(),
                    Some(path.clone()),
                    &state.workspace_folder,
                    false,
                );

                state.start_lsp(&buffer.language);

                state.lsp_open_file(&buffer.language, path, initial_text);

                state.buffer_idx = Some(state.add_buffer(buffer));
            };
        }
        Action::OpenFile(file_path) => {
            let mut path = path::PathBuf::from(file_path);
            if path.is_relative() {
                path = match std::path::absolute(path) {
                    Ok(path) => path,
                    Err(err) => {
                        error!(%err, "Failed to resolve absolute path");
                        return None;
                    }
                };
            }
            let Some(path_str) = path.to_str() else {
                error!("Failed to open file: path is not valid UTF-8");
                return None;
            };
            perform_action(Action::CreateBufferFromFile(path_str.to_string()), state);
        }
        Action::ListBuffers => {
            let mut entries: Vec<BufferListEntry> = state
                .buffers
                .iter()
                .map(|(idx, buffer)| BufferListEntry {
                    id: *idx,
                    display_name: buffer.display_name.clone().unwrap_or(idx.to_string()),
                    special: buffer.special,
                    modified: buffer.modified,
                    is_active: Some(*idx) == state.buffer_idx,
                })
                .collect();
            entries.sort_by_key(|entry| entry.id);

            return Some(serde_json::to_string(&entries).unwrap());
        }
        Action::GetActions => {
            let actions: Vec<String> = Action::VARIANTS
                .iter()
                .map(|action| action.to_string())
                .collect();

            return Some(serde_json::to_string(&actions).unwrap());
        }
        Action::FormatCurrentBuffer => {
            let (buffer, _instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            if let Some(lsp_handle) = lsp_handle {
                lsp_handle
                    .lock()
                    .unwrap()
                    .send_request_sync(
                        "textDocument/formatting".to_string(),
                        Some(LSPClientHandle::formatting_request(
                            buffer.file_path().cloned().unwrap(),
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
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            if let Some(lsp_handle) = lsp_handle {
                lsp_handle
                    .lock()
                    .unwrap()
                    .send_request_sync(
                        "textDocument/hover".to_string(),
                        Some(LSPClientHandle::hover_request(
                            buffer.file_path().cloned().unwrap(),
                            instance.cursor,
                        )),
                    )
                    .unwrap();
            }
        }
        Action::LSPCompletion => {
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            if let Some(lsp_handle) = lsp_handle {
                lsp_handle
                    .lock()
                    .unwrap()
                    .send_request_sync(
                        "textDocument/completion".to_string(),
                        Some(LSPClientHandle::completion_request(
                            buffer.file_path().cloned().unwrap(),
                            instance.cursor,
                        )),
                    )
                    .unwrap();
            }
        }
        Action::LSPSignatureHelp => {
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            if let Some(lsp_handle) = lsp_handle {
                lsp_handle
                    .lock()
                    .unwrap()
                    .send_request_sync(
                        "textDocument/signatureHelp".to_string(),
                        Some(LSPClientHandle::signature_help_request(
                            buffer.file_path().cloned().unwrap(),
                            instance.cursor,
                        )),
                    )
                    .unwrap();
            }
        }
        Action::GoToDefinition => {
            if let Some(buffer_idx) = state.buffer_idx {
                let lsp_handle = state.get_lsp_handle_for_buffer(buffer_idx);
                let (file_path, cursor) = ({
                    let (buffer, instance) = state.get_buffer_by_id(buffer_idx);
                    buffer
                        .file_path()
                        .cloned()
                        .map(|path| (path, instance.cursor))
                })?;

                let request_sent = if let Some(lsp_handle) = lsp_handle {
                    lsp_handle
                        .lock()
                        .unwrap()
                        .send_request_sync(
                            "textDocument/definition".to_string(),
                            Some(LSPClientHandle::go_to_definition_request(
                                file_path.clone(),
                                cursor,
                            )),
                        )
                        .is_ok()
                } else {
                    false
                };

                if !request_sent {
                    tracing::warn!("Failed to send LSP definitions request for {}", file_path);
                }
            }
        }
        Action::GetDefinitions => {
            if let Some(buffer_idx) = state.buffer_idx {
                let (buffer, _instance) = state.get_buffer_by_id(buffer_idx);
                if buffer.file_path().is_none() {
                    state.definitions.clear();
                    return Some("[]".to_string());
                }
            } else {
                state.definitions.clear();
                return Some("[]".to_string());
            }

            return Some(serde_json::to_string(&state.definitions).unwrap());
        }
        Action::GoToReferences => {
            if state.buffer_idx.is_some() {
                let buffer_id = state.buffer_idx.unwrap();
                let (file_path, cursor) = ({
                    let (buffer, instance) = state.get_buffer_by_id(buffer_id);
                    buffer
                        .file_path()
                        .cloned()
                        .map(|path| (path, instance.cursor))
                })?;
                let lsp_handle = state.get_lsp_handle_for_buffer(buffer_id);

                let request_sent = if let Some(lsp_handle) = lsp_handle {
                    lsp_handle
                        .lock()
                        .unwrap()
                        .send_request_sync(
                            "textDocument/references".to_string(),
                            Some(LSPClientHandle::go_to_references_request(
                                file_path.clone(),
                                cursor,
                            )),
                        )
                        .is_ok()
                } else {
                    false
                };

                if !request_sent {
                    tracing::warn!("Failed to send LSP references request for {}", file_path);
                }
            }
        }
        Action::DeletePreviousCharacter => {
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            buffer.move_cursor_left(&mut instance.selection.mark);

            let (_text, cursor) = buffer.remove_text(&instance.selection, &lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::DeleteNextCharacter => {
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            buffer.move_cursor_right(&mut instance.selection.cursor);

            let (_text, cursor) = buffer.remove_text(&instance.selection, &lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::DeleteSelection => {
            perform_action(Action::CopyToRegister, state);
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            let (_text, cursor) = buffer.remove_text(&instance.selection, &lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::Undo => {
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            if let Some(cursor) = buffer.undo(&lsp_handle) {
                instance.cursor = cursor;
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::Redo => {
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            if let Some(cursor) = buffer.redo(&lsp_handle) {
                instance.cursor = cursor;
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
                instance.column_level = instance.cursor.column;
            }
        }
        Action::AddTab => {
            let tab_width = state.preferences.tab_width;
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            let cursor =
                buffer.insert_text(&" ".repeat(tab_width), &instance.cursor, &lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
            instance.column_level = instance.cursor.column;
        }
        Action::CopyToRegister => {
            let (buffer, instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
            let selection = buffer.get_selection(&instance.selection);
            state.register = selection;
        }
        Action::CopyToClipboard => {
            let (buffer, instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());

            // Use wl-copy if available on wayland for better compatibility
            if std::env::var_os("WAYLAND_DISPLAY").is_some() && which::which("wl-copy").is_ok() {
                run_command(
                    ProgramArgs {
                        program: "wl-copy".into(),
                        args: vec![
                            "--type".to_string(),
                            "text/plain".to_string(),
                            buffer.get_selection(&instance.selection),
                        ],
                    },
                    |result, _state| {
                        if let Err(err) = result {
                            warn!(?err, "Failed to copy selection via wl-copy");
                        }
                    },
                    &state.rt,
                    state.async_handle.sender.clone(),
                    state.workspace_folder.clone(),
                );
            } else {
                let selection = buffer.get_selection(&instance.selection);

                if let Some(clipboard_ctx) = state.clipboard_ctx.as_mut() {
                    clipboard_ctx.set_contents(selection).unwrap();
                }
            }
        }
        Action::PasteFromRegister => {
            let content = state.register.clone();
            let (buffer, instance, lsp_handle) =
                state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
            let cursor = buffer.insert_text(&content, &instance.cursor, &lsp_handle, true);
            instance.cursor = cursor;
            instance.selection.cursor = instance.cursor;
            instance.selection.mark = instance.cursor;
        }
        Action::PasteFromClipboard => {
            // Use wl-paste if available on wayland for better compatibility
            if std::env::var_os("WAYLAND_DISPLAY").is_some() && which::which("wl-paste").is_ok() {
                run_command(
                    ProgramArgs {
                        program: "wl-paste".into(),
                        args: vec!["--no-newline".to_string()],
                    },
                    |result, state| {
                        let result = match result {
                            Ok(result) => result,
                            Err(err) => {
                                warn!(?err, "Failed to read clipboard via wl-paste");
                                return;
                            }
                        };

                        let (buffer, instance, lsp_handle) =
                            state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
                        let cursor =
                            buffer.insert_text(&result, &instance.cursor, &lsp_handle, true);
                        instance.cursor = cursor;
                        instance.selection.cursor = instance.cursor;
                        instance.selection.mark = instance.cursor;
                    },
                    &state.rt,
                    state.async_handle.sender.clone(),
                    state.workspace_folder.clone(),
                );
            } else {
                let content = if let Some(clipboard_ctx) = state.clipboard_ctx.as_mut() {
                    clipboard_ctx.get_contents().unwrap()
                } else {
                    String::new()
                };
                let (buffer, instance, lsp_handle) =
                    state.get_buffer_with_lsp_by_id_mut(state.buffer_idx.unwrap());
                let cursor = buffer.insert_text(&content, &instance.cursor, &lsp_handle, true);
                instance.cursor = cursor;
                instance.selection.cursor = instance.cursor;
                instance.selection.mark = instance.cursor;
            }
        }
        Action::SetSearchQuery(query) => {
            state.search_query = query;
        }
        Action::SetSearchQueryFromSelectionOrPrompt => {
            let buffer_id = state.buffer_idx?;

            let selection_text = {
                let (buffer, instance) = state.get_buffer_by_id(buffer_id);
                let (start, end) = instance.selection.in_order();
                if start != end {
                    Some(buffer.get_selection(&instance.selection))
                } else {
                    None
                }
            };

            if let Some(selection_text) = selection_text {
                state.search_query = selection_text;
                perform_action(Action::FindNextWithQuery, state);
            } else {
                perform_action(
                    Action::RunSource(
                        "dialogModalOpen(\"Enter search query\", setSearchQueryFromDialog)"
                            .to_string(),
                    ),
                    state,
                );
            }
        }
        Action::FindNextWithQuery => {
            let buffer_id = state.buffer_idx?;
            if state.search_query.is_empty() {
                return None;
            }

            let query = state.search_query.clone();
            let (buffer, instance) = state.get_buffer_by_id_mut(buffer_id);
            if let Some(selection) = buffer.find_next(&instance.cursor, &query) {
                instance.selection = selection;
                instance.cursor = selection.cursor;
            }
        }
        Action::SearchWorkspace => {
            perform_action(
                Action::RunSource("createWorkspaceSearch()".to_string()),
                state,
            );
        }
        Action::GetReferences => {
            if state.buffer_idx.is_some() {
                let buffer_id = state.buffer_idx.unwrap();
                let (buffer, _instance) = state.get_buffer_by_id(buffer_id);
                if buffer.file_path().is_none() {
                    state.references.clear();
                    return Some("[]".to_string());
                }
            } else {
                state.references.clear();
                return Some("[]".to_string());
            }

            return Some(serde_json::to_string(&state.references).unwrap());
        }
        Action::GetWorkspaceDiagnostics => {
            let mut workspace_diagnostics: Vec<WorkspaceDiagnosticEntry> = vec![];
            for (file_path, diagnostics) in &state.diagnostics {
                for diagnostic in diagnostics.diagnostics.iter() {
                    workspace_diagnostics.push(WorkspaceDiagnosticEntry {
                        file_path: file_path.clone(),
                        message: diagnostic.message.clone(),
                        severity: diagnostic_severity_label(&diagnostic.severity).to_string(),
                        source: diagnostic.source.clone(),
                        code: diagnostic.code.clone(),
                        range: diagnostic.range,
                    });
                }
            }

            return Some(serde_json::to_string(&workspace_diagnostics).unwrap());
        }
        Action::WorkspaceDiagnostics => {
            perform_action(
                Action::RunSource("createWorkspaceDiagnostics()".to_string()),
                state,
            );
        }
        Action::RunAction(action_name) => {
            if let Ok(action) = Action::from_str(action_name.trim()) {
                return perform_action(action, state);
            }
        }
        Action::OpenCommandDispatcher => {
            perform_action(
                Action::RunSource("createCommandDispatcher()".to_string()),
                state,
            );
        }
        Action::KeybindHelp => {
            let help_content = state
                .keybind_handler
                .global_keybinds
                .iter()
                .map(|keybind| keybind.definition.clone())
                .collect::<Vec<_>>()
                .join("\n");
            open_info_modal_in_rsl(state, &help_content);
        }
        Action::IncreaseFontSize => {
            state.preferences.editor_font_size += 1;
        }
        Action::DecreaseFontSize => {
            state.preferences.editor_font_size -= 1;
        }
        Action::ScrollUp => {
            let (_buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            instance.scroll.row = instance.scroll.row.saturating_sub(1);
        }
        Action::ScrollDown => {
            let (_buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
            instance.scroll.row = instance.scroll.row.saturating_add(1);
        }
        Action::Log(message) => {
            state.log_messages.push(message);
        }
        Action::SetInitRslComplete => {
            state.init_rsl_complete = true;
        }
        Action::RegisterGlobalKeybind(definition, function_id) => {
            state
                .keybind_handler
                .register_global_keybind(&definition, &function_id);
        }
        Action::RegisterBufferKeybind(buffer_id, definition, function_id) => {
            state
                .keybind_handler
                .register_buffer_keybind(buffer_id, &definition, &function_id);
        }
        Action::RegisterBufferInputHook(buffer_id, function_id) => {
            let (buffer, _instance) = state.get_buffer_by_id_mut(buffer_id);
            buffer.input_hook = Some(function_id);
        }
    };
    None
}
