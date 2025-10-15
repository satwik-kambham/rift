use std::collections::HashSet;

use crate::{
    buffer::instance::{Attribute, Cursor, Range},
    state::EditorState,
};

pub fn update_visible_lines(
    state: &mut EditorState,
    visible_lines: usize,
    max_characters: usize,
) -> Cursor {
    if state.buffer_idx.is_some() {
        let (buffer, instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
        let mut extra_segments = vec![];

        if let Some(path) = buffer.file_path.as_ref() {
            let path = path.clone();
            #[cfg(target_os = "windows")]
            let path = path.to_lowercase();

            if let Some(diagnostics) = state.diagnostics.get(&path) {
                let mut diagnostic_info = String::new();
                if diagnostics.version != 0 && diagnostics.version == buffer.version {
                    for diagnostic in &diagnostics.diagnostics {
                        if instance.cursor >= diagnostic.range.mark
                            && instance.cursor <= diagnostic.range.cursor
                        {
                            diagnostic_info.push_str(&format!(
                                "{} {} {}\n",
                                diagnostic.source, diagnostic.code, diagnostic.message
                            ));
                        }

                        extra_segments.push(Range {
                            start: buffer.byte_index_from_cursor(&diagnostic.range.mark, "\n"),
                            end: buffer.byte_index_from_cursor(&diagnostic.range.cursor, "\n"),
                            attributes: HashSet::from([Attribute::DiagnosticSeverity(
                                diagnostic.severity.clone(),
                            )]),
                        });
                    }
                }
                state.diagnostics_overlay.content = diagnostic_info;
            }
        }

        let (buffer, instance) = state.get_buffer_by_id_mut(state.buffer_idx.unwrap());
        let (lines, relative_cursor, gutter_info) = buffer.get_visible_lines(
            &mut instance.scroll,
            &instance.cursor,
            &instance.selection,
            visible_lines,
            max_characters,
            "\n".into(),
            extra_segments,
        );
        state.highlighted_text = lines;
        state.gutter_info = gutter_info;
        return relative_cursor;
    }
    Cursor { row: 0, column: 0 }
}
