use std::collections::HashMap;

use crate::{
    buffer::{
        instance::{Cursor, Range, TextAttributes, VirtualSpan},
        rope_buffer::VisibleLineParams,
    },
    lsp::types::DiagnosticSeverity,
    state::EditorState,
};

pub fn update_visible_lines(
    state: &mut EditorState,
    viewport_rows: usize,
    viewport_columns: usize,
    free_scroll: bool,
) -> Cursor {
    if let Some(buffer_idx) = state.buffer_idx {
        let (buffer, _instance) = state.get_buffer_by_id(buffer_idx);
        let mut extra_segments = vec![];
        let mut diagnostic_spans = vec![];

        if let Some(path) = buffer.file_path() {
            let path = path.clone();
            #[cfg(target_os = "windows")]
            let path = path.to_lowercase();

            if let Some(diagnostics) = state.diagnostics.get(&path)
                && diagnostics.version != 0
                && diagnostics.version == buffer.version
            {
                // Group diagnostics by start row, keeping the highest severity per row
                let mut by_row: HashMap<usize, (&DiagnosticSeverity, String)> = HashMap::new();

                for diagnostic in &diagnostics.diagnostics {
                    extra_segments.push(Range {
                        start: buffer.byte_index_from_cursor(&diagnostic.range.mark),
                        end: buffer.byte_index_from_cursor(&diagnostic.range.cursor),
                        attributes: TextAttributes::from_diagnostic_severity(&diagnostic.severity),
                    });

                    let row = diagnostic.range.mark.row;
                    match by_row.get(&row) {
                        Some((existing_sev, existing_msg)) => {
                            if diagnostic.severity > **existing_sev {
                                by_row.insert(
                                    row,
                                    (&diagnostic.severity, diagnostic.message.clone()),
                                );
                            } else if diagnostic.severity == **existing_sev {
                                let combined = format!("{} | {}", existing_msg, diagnostic.message);
                                by_row.insert(row, (existing_sev, combined));
                            }
                        }
                        None => {
                            by_row.insert(row, (&diagnostic.severity, diagnostic.message.clone()));
                        }
                    }
                }

                let max_len = viewport_columns / 2;
                for (row, (severity, message)) in &by_row {
                    let mut text = format!("  {message}");
                    if text.len() > max_len {
                        text.truncate(max_len.saturating_sub(1));
                        text.push('…');
                    }
                    let col = buffer.get_line_length(*row);
                    diagnostic_spans.push(VirtualSpan {
                        position: Cursor {
                            row: *row,
                            column: col,
                        },
                        text,
                        attributes: TextAttributes::VIRTUAL
                            | TextAttributes::from_diagnostic_severity(severity),
                    });
                }
            }
        }

        let (buffer, instance) = state.get_buffer_by_id_mut(buffer_idx);
        // Merge diagnostic virtual spans with any existing non-diagnostic spans
        instance
            .virtual_spans
            .retain(|s| !s.attributes.has_diagnostic());
        instance.virtual_spans.extend(diagnostic_spans);
        let virtual_spans = instance.virtual_spans.clone();
        let visible_line_params = VisibleLineParams {
            viewport_rows,
            viewport_columns,
            eol_sequence: "\n".into(),
        };
        let cursor = if free_scroll {
            None
        } else {
            Some(&instance.cursor)
        };
        let (lines, relative_cursor, gutter_info) = buffer.get_visible_lines(
            &mut instance.scroll,
            cursor,
            &instance.selection,
            &visible_line_params,
            extra_segments,
            &virtual_spans,
        );
        state.highlighted_text = lines;
        state.gutter_info = gutter_info;
        return relative_cursor;
    }
    Cursor { row: 0, column: 0 }
}
