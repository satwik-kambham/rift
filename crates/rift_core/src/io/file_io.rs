use anyhow::Result;
use std::{
    fs::File,
    io::{Read, Write},
};

use notify::{Event, EventKind, Result as NotifyResult};
use tracing::{info, warn};

use crate::{
    buffer::instance::{Cursor, Selection},
    state::EditorState,
};

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

/// Handles a single file event received from the watcher.
pub fn handle_file_event(file_event_result: NotifyResult<Event>, state: &mut EditorState) {
    match file_event_result {
        Ok(event) => {
            if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_))
                && event.paths.len() == 1
            {
                let Some(file_path) = event.paths.first().and_then(|path| path.to_str()) else {
                    warn!(
                        "File event missing path or invalid UTF-8: {:?}",
                        event.paths
                    );
                    return;
                };
                if let Some((buffer_id, _)) = state
                    .buffers
                    .iter()
                    .find(|(_, buf)| buf.file_path().cloned().unwrap_or_default() == file_path)
                {
                    let (buffer, instance) = state.get_buffer_by_id_mut(*buffer_id);
                    if buffer.modified {
                        warn!(
                            path = %file_path,
                            "Skipped external file change because buffer has unsaved edits"
                        );
                        return;
                    }

                    let lsp_handle = state.get_lsp_handle_for_language(&buffer.language);
                    let content = match read_file_content(file_path) {
                        Ok(content) => content,
                        Err(err) => {
                            warn!(%err, path = %file_path, "Failed to read file after change");
                            return;
                        }
                    };

                    if buffer.get_content("\n".to_string()) != content {
                        info!("Buffer modified by external process. UPDATING!");
                        buffer.reset();
                        buffer.remove_text(
                            &Selection {
                                mark: Cursor::default(),
                                cursor: Cursor {
                                    row: buffer.get_num_lines().saturating_sub(1),
                                    column: buffer
                                        .get_line_length(buffer.get_num_lines().saturating_sub(1)),
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

                        buffer.modified = false;
                    }
                }
            }
        }
        Err(e) => {
            tracing::error!("Error receiving file event: {:?}", e);
        }
    }
}
