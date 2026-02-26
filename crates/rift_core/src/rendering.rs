use crate::{
    buffer::{
        instance::{Cursor, Range, TextAttributes},
        rope_buffer::VisibleLineParams,
    },
    state::EditorState,
};

pub fn update_visible_lines(
    state: &mut EditorState,
    viewport_rows: usize,
    viewport_columns: usize,
) -> Cursor {
    if let Some(buffer_idx) = state.buffer_idx {
        let (buffer, instance) = state.get_buffer_by_id(buffer_idx);
        let mut extra_segments = vec![];

        if let Some(path) = buffer.file_path() {
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
                            start: buffer.byte_index_from_cursor(&diagnostic.range.mark),
                            end: buffer.byte_index_from_cursor(&diagnostic.range.cursor),
                            attributes: TextAttributes::from_diagnostic_severity(
                                &diagnostic.severity,
                            ),
                        });
                    }
                }
                state.diagnostics_overlay.content = diagnostic_info;
            }
        }

        let (buffer, instance) = state.get_buffer_by_id_mut(buffer_idx);
        let visible_line_params = VisibleLineParams {
            viewport_rows,
            viewport_columns,
            eol_sequence: "\n".into(),
        };
        let (lines, relative_cursor, gutter_info) = buffer.get_visible_lines(
            &mut instance.scroll,
            &instance.cursor,
            &instance.selection,
            &visible_line_params,
            extra_segments,
        );
        state.highlighted_text = lines;
        state.gutter_info = gutter_info;
        return relative_cursor;
    }
    Cursor { row: 0, column: 0 }
}
