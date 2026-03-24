use copypasta::ClipboardProvider;
use tracing::warn;

use crate::{
    concurrent::{
        AsyncPayload,
        cli::{ProgramArgs, run_command},
    },
    state::EditorState,
};

pub fn copy_to_register(state: &mut EditorState) -> Option<String> {
    let (buffer, instance) = state.get_buffer_by_id(state.buffer_idx?);
    let selection = buffer.get_selection(&instance.selection);
    state.register = selection;
    None
}

pub fn copy_to_clipboard(state: &mut EditorState) -> Option<String> {
    let buffer_idx = state.buffer_idx?;
    let (buffer, instance) = state.get_buffer_by_id(buffer_idx);

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
            |result, _state| match result {
                Ok(_) => {}
                Err(err) => {
                    warn!(?err, "Failed to copy selection via wl-copy");
                }
            },
            &state.rt_handle,
            state.async_handle.sender.clone(),
            state.workspace_folder.clone(),
        );
    } else {
        let selection = buffer.get_selection(&instance.selection);
        if let Some(clipboard_ctx) = state.clipboard_ctx.as_mut()
            && let Err(err) = clipboard_ctx.set_contents(selection)
        {
            warn!(?err, "Failed to copy to clipboard");
        }
    }
    None
}

pub fn paste_from_register(state: &mut EditorState) -> Option<String> {
    let content = state.register.clone();
    let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
    let cursor = buffer.insert_text(&content, &instance.cursor, &lsp_handle, true);
    instance.cursor = cursor;
    instance.selection.cursor = cursor;
    instance.selection.mark = cursor;
    None
}

pub fn paste_from_clipboard(state: &mut EditorState) -> Option<String> {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() && which::which("wl-paste").is_ok() {
        run_command(
            ProgramArgs {
                program: "wl-paste".into(),
                args: vec!["--no-newline".to_string()],
            },
            |result, state| {
                let result = match result {
                    Ok(AsyncPayload::Text(result)) => result,
                    Ok(AsyncPayload::Bytes(_)) => {
                        warn!("Unexpected binary clipboard data from wl-paste");
                        return;
                    }
                    Err(err) => {
                        warn!(?err, "Failed to read clipboard via wl-paste");
                        return;
                    }
                };

                let Some(buffer_idx) = state.buffer_idx else {
                    return;
                };
                let (buffer, instance, lsp_handle) =
                    state.get_buffer_with_lsp_by_id_mut(buffer_idx);
                let cursor = buffer.insert_text(&result, &instance.cursor, &lsp_handle, true);
                instance.cursor = cursor;
                instance.selection.cursor = cursor;
                instance.selection.mark = cursor;
            },
            &state.rt_handle,
            state.async_handle.sender.clone(),
            state.workspace_folder.clone(),
        );
    } else {
        let content = if let Some(clipboard_ctx) = state.clipboard_ctx.as_mut() {
            match clipboard_ctx.get_contents() {
                Ok(contents) => contents,
                Err(err) => {
                    warn!(?err, "Failed to read from clipboard");
                    String::new()
                }
            }
        } else {
            String::new()
        };
        let (buffer, instance, lsp_handle) = state.get_buffer_with_lsp_by_id_mut(state.buffer_idx?);
        let cursor = buffer.insert_text(&content, &instance.cursor, &lsp_handle, true);
        instance.cursor = cursor;
        instance.selection.cursor = cursor;
        instance.selection.mark = cursor;
    }
    None
}
