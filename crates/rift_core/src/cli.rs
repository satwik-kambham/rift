use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

use crate::{
    buffer::{instance::Language, rope_buffer::RopeBuffer},
    io::file_io,
    lsp::client::LSPClientHandle,
    state::EditorState,
};

#[derive(Parser, Debug)]
pub struct CLIArgs {
    pub path: Option<PathBuf>,
    #[arg(long, default_value_t = false, help = "Do not start language servers")]
    pub no_lsp: bool,
}

pub fn process_cli_args(
    cli_args: CLIArgs,
    state: &mut EditorState,
    lsp_handles: &mut HashMap<Language, LSPClientHandle>,
) -> Result<()> {
    state.preferences.no_lsp = cli_args.no_lsp;

    let mut path = cli_args
        .path
        .unwrap_or(std::env::current_dir().context("determining current directory")?);
    if path.try_exists().context("checking existence of path")? {
        path = path
            .canonicalize()
            .with_context(|| format!("canonicalizing {}", path.display()))?;
    } else {
        if path
            .to_str()
            .map(|p| p.ends_with(std::path::MAIN_SEPARATOR))
            .unwrap_or(false)
        {
            if !path.try_exists().context("checking existence of path")? {
                std::fs::create_dir_all(&path)
                    .with_context(|| format!("creating directory {}", path.display()))?;
            }
        } else if !path
            .parent()
            .context("resolving parent for file creation")?
            .try_exists()
            .context("checking existence of parent path")?
        {
            let parent = path
                .parent()
                .context("resolving parent for file creation")?;
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating directory {}", parent.display()))?;
            std::fs::File::create(&path)
                .with_context(|| format!("creating file {}", path.display()))?;
        }
        path = path
            .canonicalize()
            .with_context(|| format!("canonicalizing {}", path.display()))?;
    }

    if path.is_dir() {
        state.workspace_folder = path
            .into_os_string()
            .into_string()
            .map_err(|_| anyhow::anyhow!("workspace path is not valid UTF-8"))?;
    } else {
        let workspace = std::env::current_dir()
            .context("determining current directory for workspace")?
            .canonicalize()
            .context("canonicalizing workspace directory")?;
        state.workspace_folder = workspace
            .into_os_string()
            .into_string()
            .map_err(|_| anyhow::anyhow!("workspace path is not valid UTF-8"))?;
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("file path is not valid UTF-8"))?;
        let initial_text = file_io::read_file_content(path_str)
            .with_context(|| format!("reading file {}", path.display()))?;
        let buffer = RopeBuffer::new(
            initial_text.clone(),
            Some(path_str.to_string()),
            &state.workspace_folder,
            false,
        );

        if let std::collections::hash_map::Entry::Vacant(e) = lsp_handles.entry(buffer.language)
            && let Some(mut lsp_handle) = state.spawn_lsp(buffer.language)
        {
            if lsp_handle
                .init_lsp_sync(state.workspace_folder.clone())
                .is_ok()
            {
                e.insert(lsp_handle);
            } else {
                state.preferences.no_lsp = true;
            }
        }

        if let Some(lsp_handle) = lsp_handles.get(&buffer.language) {
            let language_id = match buffer.language {
                Language::Python => "python",
                Language::Rust => "rust",
                Language::Markdown => "markdown",
                Language::Dart => "dart",
                Language::Nix => "nix",
                Language::HTML => "html",
                Language::CSS => "css",
                Language::Javascript => "javascript",
                Language::Typescript => "typescript",
                Language::JSON => "json",
                Language::C => "c",
                Language::CPP => "cpp",
                Language::Vue => "vue",
                _ => "",
            };

            if (lsp_handle.initialize_capabilities["textDocumentSync"].is_number()
                || lsp_handle.initialize_capabilities["textDocumentSync"]["openClose"]
                    .as_bool()
                    .unwrap_or(false))
                && let Err(err) = lsp_handle.send_notification_sync(
                    "textDocument/didOpen".to_string(),
                    Some(LSPClientHandle::did_open_text_document(
                        path_str.to_string(),
                        language_id.to_string(),
                        initial_text,
                    )),
                )
            {
                tracing::warn!(%err, "Failed to send didOpen notification");
            }
        }

        state.buffer_idx = Some(state.add_buffer(buffer));
    }

    Ok(())
}
