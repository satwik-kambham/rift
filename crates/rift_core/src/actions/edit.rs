use serde_json::Value;

use crate::{
    buffer::instance::{Cursor, Selection},
    preferences::Preferences,
    state::{EditorState, Mode},
};

use super::{Action, perform_action};

fn trigger_chars_match(trigger_chars: &[Value], text: &str) -> bool {
    trigger_chars
        .iter()
        .any(|value| value.as_str() == Some(text))
}

pub fn insert_text_at_cursor(state: &mut EditorState, text: String) -> Option<String> {
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    let cursor = buffer.insert_text(&text, &instance.cursor, &lsp_handle, true);
    instance.cursor = cursor;
    instance.selection.cursor = cursor;
    instance.selection.mark = cursor;

    state.completion_menu.close();
    state.signature_information.content = String::new();
    None
}

pub fn insert_text_at_cursor_and_trigger_completion(
    state: &mut EditorState,
    text: String,
) -> Option<String> {
    perform_action(Action::InsertTextAtCursor(text.clone()), state);

    let buffer_idx = state.buffer_idx?;
    let lsp_handle = state.get_lsp_handle_for_buffer(buffer_idx)?;

    let (completion_triggers, signature_triggers) = {
        let lsp_handle = lsp_handle.lock().unwrap();
        let completion_triggers =
            lsp_handle.initialize_capabilities["completionProvider"]["triggerCharacters"]
                .as_array()
                .cloned()
                .unwrap_or_default();
        let signature_triggers =
            lsp_handle.initialize_capabilities["signatureHelpProvider"]["triggerCharacters"]
                .as_array()
                .cloned()
                .unwrap_or_default();
        (completion_triggers, signature_triggers)
    };

    let text_is_alpha = text
        .chars()
        .next()
        .map(|ch| text.len() == 1 && ch.is_ascii_alphabetic())
        .unwrap_or(false);
    let completion_triggered = text_is_alpha || trigger_chars_match(&completion_triggers, &text);
    let signature_triggered = trigger_chars_match(&signature_triggers, &text);

    if completion_triggered {
        perform_action(Action::LSPCompletion, state);
    }
    if signature_triggered {
        perform_action(Action::LSPSignatureHelp, state);
    }
    None
}

pub fn insert_text(state: &mut EditorState, text: String, cursor: Cursor) -> Option<String> {
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    let cursor = buffer.insert_text(&text, &cursor, &lsp_handle, true);
    instance.sync_cursor(cursor);
    None
}

pub fn delete_text(state: &mut EditorState, selection: Selection) -> Option<String> {
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    let (_text, cursor) = buffer.remove_text(&selection, &lsp_handle, true);
    instance.sync_cursor(cursor);
    None
}

pub fn insert_new_line_at_cursor(state: &mut EditorState) -> Option<String> {
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    instance.cursor = instance.selection.cursor;
    let indent_size = buffer.get_indentation_level(instance.cursor.row);
    let cursor = buffer.insert_text("\n", &instance.cursor, &lsp_handle, true);
    instance.cursor = cursor;
    instance.selection.cursor = cursor;
    instance.selection.mark = cursor;
    instance.selection = buffer.add_indentation(&instance.selection, indent_size, &lsp_handle);
    instance.cursor = instance.selection.cursor;
    instance.column_level = instance.cursor.column;
    None
}

pub fn enter_insert_mode(state: &mut EditorState) -> Option<String> {
    if matches!(state.mode, Mode::Normal) {
        state.mode = Mode::Insert;
        perform_action(Action::LSPCompletion, state);
        perform_action(Action::LSPSignatureHelp, state);
    }
    None
}

pub fn quit_insert_mode(state: &mut EditorState) -> Option<String> {
    state.mode = Mode::Normal;
    state.signature_information.content = String::new();
    None
}

pub fn delete_selection_and_enter_insert_mode(state: &mut EditorState) -> Option<String> {
    if matches!(state.mode, Mode::Normal) {
        perform_action(Action::DeleteSelection, state);
        perform_action(Action::EnterInsertMode, state);
    }
    None
}

pub fn add_new_line_below_and_enter_insert_mode(state: &mut EditorState) -> Option<String> {
    if matches!(state.mode, Mode::Normal) {
        let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
        instance.cursor = instance.selection.cursor;
        let indent_size = buffer.get_indentation_level(instance.cursor.row);
        buffer.move_cursor_line_end(&mut instance.cursor);
        let cursor = buffer.insert_text("\n", &instance.cursor, &lsp_handle, true);
        instance.cursor = cursor;
        instance.selection.cursor = cursor;
        instance.selection.mark = cursor;
        instance.selection = buffer.add_indentation(&instance.selection, indent_size, &lsp_handle);
        instance.cursor = instance.selection.cursor;
        instance.column_level = instance.cursor.column;
        perform_action(Action::EnterInsertMode, state);
    }
    None
}

pub fn add_indent(state: &mut EditorState) -> Option<String> {
    let tab_width = state.preferences.tab_width;
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    instance.selection = buffer.add_indentation(&instance.selection, tab_width, &lsp_handle);
    instance.cursor = instance.selection.cursor;
    instance.column_level = instance.cursor.column;
    None
}

pub fn remove_indent(state: &mut EditorState) -> Option<String> {
    let tab_width = state.preferences.tab_width;
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    instance.selection = buffer.remove_indentation(&instance.selection, tab_width, &lsp_handle);
    instance.cursor = instance.selection.cursor;
    instance.column_level = instance.cursor.column;
    None
}

pub fn toggle_comment(state: &mut EditorState) -> Option<String> {
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    let comment_token = Preferences::get_comment_token(buffer.language);
    instance.selection = buffer.toggle_comment(&instance.selection, comment_token, &lsp_handle);
    instance.cursor = instance.selection.cursor;
    instance.column_level = instance.cursor.column;
    None
}

pub fn delete_previous_character(state: &mut EditorState) -> Option<String> {
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    instance.selection.cursor = instance.cursor;
    instance.selection.mark = instance.cursor;
    buffer.move_cursor_left(&mut instance.selection.mark);

    let (_text, cursor) = buffer.remove_text(&instance.selection, &lsp_handle, true);
    instance.sync_cursor(cursor);
    None
}

pub fn delete_next_character(state: &mut EditorState) -> Option<String> {
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    instance.selection.cursor = instance.cursor;
    instance.selection.mark = instance.cursor;
    buffer.move_cursor_right(&mut instance.selection.cursor);

    let (_text, cursor) = buffer.remove_text(&instance.selection, &lsp_handle, true);
    instance.sync_cursor(cursor);
    None
}

pub fn delete_selection(state: &mut EditorState) -> Option<String> {
    perform_action(Action::CopyToRegister, state);
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    let (_text, cursor) = buffer.remove_text(&instance.selection, &lsp_handle, true);
    instance.sync_cursor(cursor);
    None
}

pub fn undo(state: &mut EditorState) -> Option<String> {
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    if let Some(cursor) = buffer.undo(&lsp_handle) {
        instance.sync_cursor(cursor);
    }
    None
}

pub fn redo(state: &mut EditorState) -> Option<String> {
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    if let Some(cursor) = buffer.redo(&lsp_handle) {
        instance.sync_cursor(cursor);
    }
    None
}

pub fn add_tab(state: &mut EditorState) -> Option<String> {
    let tab_width = state.preferences.tab_width;
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    let cursor = buffer.insert_text(&" ".repeat(tab_width), &instance.cursor, &lsp_handle, true);
    instance.sync_cursor(cursor);
    None
}

pub fn select(state: &mut EditorState, selection: Selection) -> Option<String> {
    let (_buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    instance.selection = selection;
    instance.cursor = selection.cursor;
    instance.column_level = selection.cursor.column;
    None
}

pub fn select_and_extend_current_line(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    instance.selection = buffer.select_line(&instance.selection);
    instance.cursor = instance.selection.cursor;
    instance.column_level = instance.cursor.column;
    None
}

pub fn select_buffer(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    let end_row = buffer.get_num_lines().saturating_sub(1);
    let end_column = buffer.get_line_length(end_row);
    instance.selection = Selection {
        mark: Cursor { row: 0, column: 0 },
        cursor: Cursor {
            row: end_row,
            column: end_column,
        },
    };
    instance.cursor = instance.selection.cursor;
    instance.column_level = instance.cursor.column;
    None
}

pub fn select_till_end_of_word(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    instance.selection.mark = instance.selection.cursor;
    instance.selection = buffer.select_word(&instance.selection);
    instance.cursor = instance.selection.cursor;
    instance.column_level = instance.cursor.column;
    None
}

pub fn extend_select_till_end_of_word(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    instance.selection = buffer.select_word(&instance.selection);
    instance.cursor = instance.selection.cursor;
    instance.column_level = instance.cursor.column;
    None
}

pub fn insert_buffer_input(state: &mut EditorState, text: String) -> Option<String> {
    let (buffer, _instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    buffer.input.push_str(&text);
    if let Some(function_id) = &buffer.input_hook {
        perform_action(
            Action::RunSource(format!("runFunctionById(\"{}\")", function_id)),
            state,
        );
    }
    None
}

pub fn set_buffer_input(state: &mut EditorState, buffer_id: u32, text: String) -> Option<String> {
    let (buffer, _instance) = state.get_buffer_by_id_mut(buffer_id);
    buffer.input = text;
    if let Some(function_id) = &buffer.input_hook {
        perform_action(
            Action::RunSource(format!("runFunctionById(\"{}\")", function_id)),
            state,
        );
    }
    None
}
