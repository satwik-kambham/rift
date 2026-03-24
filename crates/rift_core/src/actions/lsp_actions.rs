use tracing::warn;

use crate::{lsp::client::LSPClientHandle, state::EditorState};

use super::{Action, perform_action};

pub fn hover(state: &mut EditorState) -> Option<String> {
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    let lsp_handle = lsp_handle?;
    let file_path = buffer.file_path()?.clone();
    if let Err(err) = lsp_handle.lock().unwrap().send_request_sync(
        "textDocument/hover".to_string(),
        Some(LSPClientHandle::hover_request(file_path, instance.cursor)),
    ) {
        warn!(%err, "Failed to send LSP hover request");
    }
    None
}

pub fn completion(state: &mut EditorState) -> Option<String> {
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    let lsp_handle = lsp_handle?;
    let file_path = buffer.file_path()?.clone();
    if let Err(err) = lsp_handle.lock().unwrap().send_request_sync(
        "textDocument/completion".to_string(),
        Some(LSPClientHandle::completion_request(
            file_path,
            instance.cursor,
        )),
    ) {
        warn!(%err, "Failed to send LSP completion request");
    }
    None
}

pub fn signature_help(state: &mut EditorState) -> Option<String> {
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    let lsp_handle = lsp_handle?;
    let file_path = buffer.file_path()?.clone();
    if let Err(err) = lsp_handle.lock().unwrap().send_request_sync(
        "textDocument/signatureHelp".to_string(),
        Some(LSPClientHandle::signature_help_request(
            file_path,
            instance.cursor,
        )),
    ) {
        warn!(%err, "Failed to send LSP signature help request");
    }
    None
}

pub fn go_to_definition(state: &mut EditorState) -> Option<String> {
    let buffer_idx = state.buffer_idx?;
    let lsp_handle = state.get_lsp_handle_for_buffer(buffer_idx);
    let (file_path, cursor) = {
        let (buffer, instance) = state.get_buffer_by_id(buffer_idx);
        buffer
            .file_path()
            .cloned()
            .map(|path| (path, instance.cursor))
    }?;

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
        warn!("Failed to send LSP definitions request for {}", file_path);
    }
    None
}

pub fn get_definitions(state: &mut EditorState) -> Option<String> {
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

    Some(serde_json::to_string(&state.definitions).unwrap())
}

pub fn go_to_references(state: &mut EditorState) -> Option<String> {
    let buffer_id = state.buffer_idx?;
    let (file_path, cursor) = {
        let (buffer, instance) = state.get_buffer_by_id(buffer_id);
        buffer
            .file_path()
            .cloned()
            .map(|path| (path, instance.cursor))
    }?;
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
        warn!("Failed to send LSP references request for {}", file_path);
    }
    None
}

pub fn get_references(state: &mut EditorState) -> Option<String> {
    if let Some(buffer_id) = state.buffer_idx {
        let (buffer, _instance) = state.get_buffer_by_id(buffer_id);
        if buffer.file_path().is_none() {
            state.references.clear();
            return Some("[]".to_string());
        }
    } else {
        state.references.clear();
        return Some("[]".to_string());
    }

    Some(serde_json::to_string(&state.references).unwrap())
}

pub fn format_current_buffer(state: &mut EditorState) -> Option<String> {
    let (buffer, _instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    if let Some(lsp_handle) = lsp_handle {
        let file_path = buffer.file_path()?.clone();
        if let Err(err) = lsp_handle.lock().unwrap().send_request_sync(
            "textDocument/formatting".to_string(),
            Some(LSPClientHandle::formatting_request(
                file_path,
                state.preferences.tab_width,
            )),
        ) {
            warn!(%err, "Failed to send LSP formatting request");
        }
    }
    None
}

pub fn workspace_diagnostics(state: &mut EditorState) -> Option<String> {
    perform_action(
        Action::RunSource("createWorkspaceDiagnostics()".to_string()),
        state,
    );
    None
}
