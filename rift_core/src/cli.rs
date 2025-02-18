use std::{collections::HashMap, path::PathBuf};

use clap::Parser;

use crate::{
    buffer::{instance::Language, line_buffer::LineBuffer},
    io::file_io,
    lsp::client::LSPClientHandle,
    state::EditorState,
};

/// CLI Arguments
#[derive(Parser, Debug)]
pub struct CLIArgs {
    pub path: Option<PathBuf>,
}

pub fn process_cli_args(
    cli_args: CLIArgs,
    state: &mut EditorState,
    lsp_handles: &mut HashMap<Language, LSPClientHandle>,
) {
    if let Some(path) = cli_args.path {
        let mut path = path;
        if path.is_relative() {
            path = std::path::absolute(path).unwrap();
        }
        if path.is_dir() {
            state.workspace_folder = path.into_os_string().into_string().unwrap();
        } else {
            state.workspace_folder = path.parent().unwrap().to_str().unwrap().to_string();
            let initial_text = file_io::read_file_content(path.to_str().unwrap()).unwrap();
            let buffer = LineBuffer::new(
                initial_text.clone(),
                Some(path.to_str().unwrap().to_string()),
            );

            if let std::collections::hash_map::Entry::Vacant(e) = lsp_handles.entry(buffer.language)
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
                            path.to_str().unwrap().to_string(),
                            initial_text,
                        )),
                    )
                    .unwrap();
            }

            state.buffer_idx = Some(state.add_buffer(buffer));
        }
    }
}
