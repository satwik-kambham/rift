mod audio_actions;
mod buffer;
mod clipboard;
mod cursor;
mod edit;
mod lsp_actions;
mod search;

use std::str::FromStr;

use serde_json::json;
use strum::{EnumIter, EnumMessage, EnumString, VariantNames};

use crate::{
    audio,
    buffer::instance::{Cursor, Selection, TextAttributes, VirtualSpan},
    lsp::types::DiagnosticSeverity,
    state::EditorState,
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

fn diagnostic_severity_label(severity: &DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Hint => "Hint",
        DiagnosticSeverity::Information => "Information",
        DiagnosticSeverity::Warning => "Warning",
        DiagnosticSeverity::Error => "Error",
    }
}

pub fn open_info_modal_in_rsl(state: &mut EditorState, content: &str) {
    let serialized = rsl_string_literal(content);
    perform_action(
        Action::RunSource(format!("infoModalOpen({})", serialized)),
        state,
    );
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
    SelectBuffer,
    SelectTillEndOfWord,
    ExtendSelectTillEndOfWord,
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
    Tts(String),
    TtsBuffer,
    #[strum(disabled)]
    StartTranscription(audio::TranscriptionCallback),
    StopTranscription,
    InsertTranscription,
    AddTestVirtualSpans,
    ClearVirtualSpans,
}

pub fn perform_action(action: Action, state: &mut EditorState) -> Option<String> {
    match action {
        // Buffer management
        Action::Quit => {
            state.quit = true;
        }
        Action::SetBufferContent(buffer_id, content) => {
            buffer::set_buffer_content(state, buffer_id, content);
        }
        Action::GetBufferInput(buffer_id) => {
            return buffer::get_buffer_input(state, buffer_id);
        }
        Action::SetActiveBuffer(buffer_id) => {
            buffer::set_active_buffer(state, buffer_id);
        }
        Action::GetActiveBuffer => {
            return buffer::get_active_buffer(state);
        }
        Action::CycleNextBuffer => {
            buffer::cycle_next_buffer(state);
        }
        Action::CyclePreviousBuffer => {
            buffer::cycle_previous_buffer(state);
        }
        Action::CloseCurrentBuffer => {
            buffer::close_current_buffer(state);
        }
        Action::SaveCurrentBuffer => {
            return buffer::save_current_buffer(state);
        }
        Action::RunCurrentBuffer => {
            return buffer::run_current_buffer(state);
        }
        Action::CreateSpecialBuffer(display_name) => {
            return buffer::create_special_buffer(state, display_name);
        }
        Action::CreateBufferFromFile(path) => {
            return buffer::create_buffer_from_file(state, path);
        }
        Action::OpenFile(file_path) => {
            return buffer::open_file(state, file_path);
        }
        Action::ListBuffers => {
            return buffer::list_buffers(state);
        }

        // Text editing
        Action::InsertBufferInput(text) => {
            return edit::insert_buffer_input(state, text);
        }
        Action::SetBufferInput(buffer_id, text) => {
            return edit::set_buffer_input(state, buffer_id, text);
        }
        Action::InsertTextAtCursor(text) => {
            return edit::insert_text_at_cursor(state, text);
        }
        Action::InsertTextAtCursorAndTriggerCompletion(text) => {
            return edit::insert_text_at_cursor_and_trigger_completion(state, text);
        }
        Action::InsertSpace => {
            return edit::insert_text_at_cursor(state, " ".to_string());
        }
        Action::InsertText(text, cursor) => {
            return edit::insert_text(state, text, cursor);
        }
        Action::DeleteText(selection) => {
            return edit::delete_text(state, selection);
        }
        Action::InsertNewLineAtCursor => {
            return edit::insert_new_line_at_cursor(state);
        }
        Action::EnterInsertMode => {
            return edit::enter_insert_mode(state);
        }
        Action::QuitInsertMode => {
            return edit::quit_insert_mode(state);
        }
        Action::DeleteSelectionAndEnterInsertMode => {
            return edit::delete_selection_and_enter_insert_mode(state);
        }
        Action::AddNewLineBelowAndEnterInsertMode => {
            return edit::add_new_line_below_and_enter_insert_mode(state);
        }
        Action::AddIndent => {
            return edit::add_indent(state);
        }
        Action::RemoveIndent => {
            return edit::remove_indent(state);
        }
        Action::ToggleComment => {
            return edit::toggle_comment(state);
        }
        Action::DeletePreviousCharacter => {
            return edit::delete_previous_character(state);
        }
        Action::DeleteNextCharacter => {
            return edit::delete_next_character(state);
        }
        Action::DeleteSelection => {
            return edit::delete_selection(state);
        }
        Action::AddTab => {
            return edit::add_tab(state);
        }
        Action::Undo => {
            return edit::undo(state);
        }
        Action::Redo => {
            return edit::redo(state);
        }

        // Selection
        Action::Select(selection) => {
            return edit::select(state, selection);
        }
        Action::SelectAndExtendCurrentLine => {
            return edit::select_and_extend_current_line(state);
        }
        Action::SelectBuffer => {
            return edit::select_buffer(state);
        }
        Action::SelectTillEndOfWord => {
            return edit::select_till_end_of_word(state);
        }
        Action::ExtendSelectTillEndOfWord => {
            return edit::extend_select_till_end_of_word(state);
        }

        // Cursor movement
        Action::MoveCursorDown => {
            return cursor::move_down(state);
        }
        Action::MoveCursorUp => {
            return cursor::move_up(state);
        }
        Action::MoveCursorLeft => {
            return cursor::move_left(state);
        }
        Action::MoveCursorRight => {
            return cursor::move_right(state);
        }
        Action::ExtendCursorDown => {
            return cursor::extend_down(state);
        }
        Action::ExtendCursorUp => {
            return cursor::extend_up(state);
        }
        Action::ExtendCursorLeft => {
            return cursor::extend_left(state);
        }
        Action::ExtendCursorRight => {
            return cursor::extend_right(state);
        }
        Action::MoveCursorLineStart => {
            return cursor::move_line_start(state);
        }
        Action::MoveCursorLineEnd => {
            return cursor::move_line_end(state);
        }
        Action::ExtendCursorLineStart => {
            return cursor::extend_line_start(state);
        }
        Action::ExtendCursorLineEnd => {
            return cursor::extend_line_end(state);
        }
        Action::GoToBufferStart => {
            return cursor::go_to_buffer_start(state);
        }
        Action::GoToBufferEnd => {
            return cursor::go_to_buffer_end(state);
        }
        Action::Unselect => {
            return cursor::unselect(state);
        }
        Action::ScrollUp => {
            return cursor::scroll_up(state);
        }
        Action::ScrollDown => {
            return cursor::scroll_down(state);
        }

        // LSP
        Action::LSPHover => {
            return lsp_actions::hover(state);
        }
        Action::LSPCompletion => {
            return lsp_actions::completion(state);
        }
        Action::LSPSignatureHelp => {
            return lsp_actions::signature_help(state);
        }
        Action::GoToDefinition => {
            return lsp_actions::go_to_definition(state);
        }
        Action::GetDefinitions => {
            return lsp_actions::get_definitions(state);
        }
        Action::GoToReferences => {
            return lsp_actions::go_to_references(state);
        }
        Action::GetReferences => {
            return lsp_actions::get_references(state);
        }
        Action::FormatCurrentBuffer => {
            return lsp_actions::format_current_buffer(state);
        }
        Action::WorkspaceDiagnostics => {
            return lsp_actions::workspace_diagnostics(state);
        }

        // Clipboard
        Action::CopyToRegister => {
            return clipboard::copy_to_register(state);
        }
        Action::CopyToClipboard => {
            return clipboard::copy_to_clipboard(state);
        }
        Action::PasteFromRegister => {
            return clipboard::paste_from_register(state);
        }
        Action::PasteFromClipboard => {
            return clipboard::paste_from_clipboard(state);
        }

        // Search
        Action::SetSearchQuery(query) => {
            search::set_search_query(state, query);
        }
        Action::SetSearchQueryFromSelectionOrPrompt => {
            return search::set_search_query_from_selection_or_prompt(state);
        }
        Action::FindNextWithQuery => {
            return search::find_next_with_query(state);
        }
        Action::SearchWorkspace => {
            search::search_workspace(state);
        }

        // Audio
        Action::Tts(text) => {
            audio_actions::tts(state, text);
        }
        Action::TtsBuffer => {
            return audio_actions::tts_buffer(state);
        }
        Action::StartTranscription(callback) => {
            return audio_actions::start_transcription(state, callback);
        }
        Action::StopTranscription => {
            audio_actions::stop_transcription(state);
        }
        Action::InsertTranscription => {
            return audio_actions::insert_transcription(state);
        }

        // Misc
        Action::RunSource(source) => {
            if let Err(err) = state.rsl_sender.try_send(source) {
                tracing::warn!(%err, "Failed to send RSL source");
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
        Action::GetActions => {
            let actions: Vec<String> = Action::VARIANTS
                .iter()
                .map(|action| action.to_string())
                .collect();
            return Some(serde_json::to_string(&actions).unwrap());
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
        Action::AddTestVirtualSpans => {
            if let Some(buffer_idx) = state.buffer_idx {
                let (buffer, instance) = state.get_buffer_by_id_mut(buffer_idx);
                let cursor = instance.cursor;
                let line_end_col = buffer.get_line_length(cursor.row);
                instance.set_virtual_spans(vec![
                    VirtualSpan {
                        position: cursor,
                        text: " /* ghost at cursor */".into(),
                        attributes: TextAttributes::HIGHLIGHT_GRAY | TextAttributes::VIRTUAL,
                    },
                    VirtualSpan {
                        position: Cursor {
                            row: cursor.row,
                            column: line_end_col,
                        },
                        text: " // end-of-line hint".into(),
                        attributes: TextAttributes::HIGHLIGHT_GRAY | TextAttributes::VIRTUAL,
                    },
                ]);
            }
        }
        Action::ClearVirtualSpans => {
            if let Some(buffer_idx) = state.buffer_idx {
                let (_buffer, instance) = state.get_buffer_by_id_mut(buffer_idx);
                instance.clear_virtual_spans();
            }
        }
    };
    None
}
