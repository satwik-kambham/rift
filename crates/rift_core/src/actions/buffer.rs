use std::path;

use tracing::{error, warn};

use crate::{
    buffer::rope_buffer::RopeBuffer, io::file_io, lsp::client::LSPClientHandle, state::EditorState,
};

use super::{Action, BufferListEntry, perform_action};

pub fn set_buffer_content(state: &mut EditorState, buffer_id: u32, content: String) {
    let (buffer, _instance) = state.get_buffer_by_id_mut(buffer_id);
    buffer.set_content(content);
}

pub fn get_buffer_input(state: &mut EditorState, buffer_id: u32) -> Option<String> {
    let (buffer, _instance) = state.get_buffer_by_id_mut(buffer_id);
    Some(buffer.input.clone())
}

pub fn set_active_buffer(state: &mut EditorState, buffer_id: u32) {
    state.buffer_idx = Some(buffer_id);
}

pub fn get_active_buffer(state: &EditorState) -> Option<String> {
    Some(
        state
            .buffer_idx
            .map(|id| id.to_string())
            .unwrap_or_default(),
    )
}

pub fn cycle_next_buffer(state: &mut EditorState) {
    state.cycle_buffer(false, true);
}

pub fn cycle_previous_buffer(state: &mut EditorState) {
    state.cycle_buffer(true, true);
}

pub fn close_current_buffer(state: &mut EditorState) {
    if let Some(buffer_id) = state.buffer_idx {
        state.remove_buffer(buffer_id);
    }
}

pub fn save_current_buffer(state: &mut EditorState) -> Option<String> {
    let line_ending = state.preferences.line_ending.clone();
    let (buffer, _instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
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
    None
}

pub fn run_current_buffer(state: &mut EditorState) -> Option<String> {
    let (buffer, _instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    let source = buffer.get_content("\n".to_string());
    perform_action(Action::RunSource(source), state);
    None
}

pub fn create_special_buffer(state: &mut EditorState, display_name: String) -> Option<String> {
    let mut buffer = RopeBuffer::new(String::new(), None, &state.workspace_folder, true);
    if !display_name.is_empty() {
        buffer.display_name = Some(display_name);
    }
    let buffer_id = state.add_buffer(buffer);
    Some(buffer_id.to_string())
}

pub fn create_buffer_from_file(state: &mut EditorState, path: String) -> Option<String> {
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
    None
}

pub fn open_file(state: &mut EditorState, file_path: String) -> Option<String> {
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
    None
}

pub fn list_buffers(state: &mut EditorState) -> Option<String> {
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

    Some(serde_json::to_string(&entries).unwrap())
}
