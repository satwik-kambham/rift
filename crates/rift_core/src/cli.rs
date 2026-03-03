use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use crate::{buffer::rope_buffer::RopeBuffer, io::file_io, state::EditorState};

#[derive(Parser, Debug)]
pub struct CLIArgs {
    pub path: Option<PathBuf>,
    #[arg(long, default_value_t = false, help = "Do not start language servers")]
    pub no_lsp: bool,
    #[arg(long, default_value_t = false, help = "Do not start audio services")]
    pub no_audio: bool,
}

pub fn process_cli_args(cli_args: CLIArgs, state: &mut EditorState) -> Result<()> {
    state.preferences.no_lsp = cli_args.no_lsp;
    state.preferences.no_audio = cli_args.no_audio;

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

        state.start_lsp(&buffer.language);

        state.lsp_open_file(&buffer.language, path_str.to_string(), initial_text);

        state.buffer_idx = Some(state.add_buffer(buffer));
    }

    Ok(())
}
