use anyhow::Result;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path,
};

use notify::{Event, EventKind, Result as NotifyResult};

use crate::{
    buffer::instance::{Cursor, Language, Selection},
    lsp::client::LSPClientHandle,
    state::EditorState,
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FolderEntry {
    pub path: String,
    pub is_dir: bool,
    pub name: String,
    pub extension: String,
    pub children: Option<Vec<FolderEntry>>,
}

impl PartialOrd for FolderEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FolderEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Prioritize is_dir, then name
        other
            .is_dir
            .cmp(&self.is_dir)
            .then_with(|| self.name.to_lowercase().cmp(&other.name.to_lowercase()))
    }
}

/// Read file at path to string
pub fn read_file_content(path: &str) -> Result<String> {
    let mut f = File::open(path)?;
    let mut buf = String::new();

    let _ = f.read_to_string(&mut buf)?;

    Ok(buf)
}

/// Override file at path with new content
pub fn override_file_content(path: &str, buf: String) -> Result<()> {
    let mut f = File::create(path)?;
    f.write_all(buf.as_bytes())?;

    Ok(())
}

/// Create directory at path (recursively)
pub fn create_directory(path: &str) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

/// Create file at path
pub fn create_file(path: &str) -> Result<()> {
    override_file_content(path, "".into())?;
    Ok(())
}

/// Delete file at path
pub fn delete_file(path: &str) -> Result<()> {
    fs::remove_file(path)?;
    Ok(())
}

/// Delete directory with all its contents
pub fn delete_directory_recursively(path: &str) -> Result<()> {
    fs::remove_dir_all(path)?;
    Ok(())
}

/// Rename file or directory
pub fn rename_file_or_directory(path: &str, to: &str) -> Result<()> {
    let mut new_path = path::PathBuf::from(path);
    new_path.pop();
    new_path.push(to);
    fs::rename(path, new_path)?;
    Ok(())
}

/// Duplicate file and append _copy to new file
pub fn duplicate_file(path: &str) -> Result<()> {
    let mut new_path = path::PathBuf::from(path);
    if let Some(stem) = new_path.file_stem() {
        if let Some(extension) = new_path.extension() {
            new_path.set_file_name(format!(
                "{}_copy.{}",
                stem.to_string_lossy(),
                extension.to_string_lossy()
            ));
        } else {
            new_path.set_file_name(format!("{}_copy", stem.to_string_lossy()));
        }
    }
    Ok(())
}

/// Move file or directory to new path
pub fn move_file_or_directory(path: &str, to: &str) -> Result<()> {
    fs::rename(path, to)?;
    Ok(())
}

/// Get all items in folder
pub fn get_directory_entries(path: &str) -> Result<Vec<FolderEntry>> {
    let mut entries: Vec<FolderEntry> = vec![];
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        entries.push(FolderEntry {
            path: path::absolute(entry.path())
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            is_dir: entry.metadata()?.is_dir(),
            name: entry
                .path()
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap()
                .to_string(),
            extension: entry
                .path()
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap()
                .to_string(),
            children: None,
        });
    }
    entries.sort();
    Ok(entries)
}

/// Handles a single file event received from the watcher.
pub fn handle_file_event(
    file_event_result: NotifyResult<Event>,
    state: &mut EditorState,
    lsp_handles: &mut HashMap<Language, LSPClientHandle>,
) {
    match file_event_result {
        Ok(event) => {
            if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
                if event.paths.len() == 1 {
                    let file_path = event.paths.first().unwrap().to_str().unwrap();
                    if let Some((buffer_id, _)) = state
                        .buffers
                        .iter()
                        .find(|(_, buf)| buf.file_path.as_ref().unwrap() == file_path)
                    {
                        let (buffer, instance) = state.get_buffer_by_id_mut(*buffer_id);
                        let lsp_handle = lsp_handles.get_mut(&buffer.language);
                        let content = read_file_content(file_path).unwrap();

                        if buffer.get_content("\n".to_string()) != content {
                            tracing::info!("Buffer modified by external process. UPDATING!");
                            buffer.reset();
                            buffer.remove_text(
                                &Selection {
                                    mark: Cursor::default(),
                                    cursor: Cursor {
                                        row: buffer.get_num_lines().saturating_sub(1),
                                        column: buffer.get_line_length(
                                            buffer.get_num_lines().saturating_sub(1),
                                        ),
                                    },
                                },
                                &lsp_handle,
                                false,
                            );
                            buffer.insert_text(&content, &Cursor::default(), &lsp_handle, false);

                            instance.selection = Selection::default();

                            let buffer_end = Cursor {
                                row: buffer.get_num_lines().saturating_sub(1),
                                column: buffer
                                    .get_line_length(buffer.get_num_lines().saturating_sub(1)),
                            };

                            if instance.cursor > buffer_end {
                                instance.cursor = buffer_end;
                            }
                            instance.column_level = instance.cursor.column;
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("Error receiving file event: {:?}", e);
        }
    }
}
