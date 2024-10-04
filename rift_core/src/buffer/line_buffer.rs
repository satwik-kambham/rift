use std::cmp::{max, min};

use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use super::instance::{Cursor, GutterInfo};

/// Text buffer implementation as a list of lines
pub struct LineBuffer {
    pub file_path: Option<String>,
    pub lines: Vec<String>,
    highlighter: Highlighter,
    language_config: HighlightConfiguration,
}

pub type HighlightedLine = Vec<Vec<(String, Option<usize>)>>;

impl LineBuffer {
    /// Create a line buffer
    pub fn new(initial_text: String, file_path: Option<String>) -> Self {
        // Split string at line endings and collect
        // into a vector of strings
        let mut lines: Vec<String> = initial_text.lines().map(String::from).collect();

        // We always want an extra empty line at
        // the end of the buffer / file
        if let Some(last) = lines.last() {
            if !last.is_empty() {
                // Last line is not empty
                lines.push("".into())
            }
        } else {
            // The buffer is empty
            lines.push("".into());
        }

        // Syntax highlighter
        let highlighter = Highlighter::new();
        let highlight_names = [
            "attribute",
            "constant",
            "function.builtin",
            "function",
            "keyword",
            "operator",
            "property",
            "punctuation",
            "punctuation.bracket",
            "punctuation.delimiter",
            "string",
            "string.special",
            "tag",
            "type",
            "type.builtin",
            "variable",
            "variable.builtin",
            "variable.parameter",
        ];
        let mut language_config = HighlightConfiguration::new(
            tree_sitter_rust::LANGUAGE.into(),
            "rust",
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            "",
        )
        .unwrap();
        language_config.configure(&highlight_names);
        println!("Highlight Names: {:#?}", language_config.names());

        Self {
            file_path,
            lines,
            highlighter,
            language_config,
        }
    }

    /// Get text buffer content as a string
    /// with the desired EOL sequence
    pub fn get_content(&self, eol_sequence: String) -> String {
        self.lines.join(&eol_sequence)
    }

    pub fn get_visible_lines(
        &mut self,
        scroll: &mut Cursor,
        cursor: &Cursor,
        visible_lines: usize,
        max_characters: usize,
    ) -> (HighlightedLine, Cursor, Vec<GutterInfo>) {
        let mut lines = vec![];
        let mut gutter_info = vec![];
        let mut relative_cursor = Cursor {
            row: 0,
            column: cursor.column,
        };
        let mut range_start = scroll.row;
        let mut range_end = range_start + visible_lines + 3;
        let mut start = 0;
        let mut cursor_idx: usize = 0;
        let mut start_byte = 0;
        let mut highlight_type: Option<usize> = None;

        // Calculate range
        if cursor < scroll {
            range_start = cursor.row.saturating_sub(3);
            range_end = range_start + visible_lines;
        } else if cursor.row >= scroll.row + visible_lines {
            range_end = cursor.row + 3;
            range_start = range_end.saturating_sub(visible_lines);
        }

        // Calculate start byte
        for line in self.lines.get(..range_start).unwrap() {
            start_byte += line.len();
        }

        // Calculate gutter info
        for (line_idx, line) in self
            .lines
            .get(range_start..range_end)
            .unwrap_or(&self.lines[range_start..])
            .iter()
            .enumerate()
        {
            while start < line.len() {
                let end = (start + max_characters).min(line.len());
                // lines.push(line[start..end].to_string());
                gutter_info.push(GutterInfo {
                    start: Cursor {
                        row: range_start + line_idx,
                        column: start,
                    },
                    end,
                    wrapped: start != 0,
                    wrap_end: end == line.len(),
                    start_byte,
                    end_byte: start_byte + end - start,
                });

                start_byte += end - start;
                start = end;
            }

            if line.is_empty() {
                // lines.push("".to_string());
                gutter_info.push(GutterInfo {
                    start: Cursor {
                        row: range_start + line_idx,
                        column: 0,
                    },
                    end: 0,
                    wrapped: false,
                    wrap_end: true,
                    start_byte,
                    end_byte: start_byte,
                });
            }

            start = 0;
        }

        // Calculate relative cursor position
        for line_info in &gutter_info {
            if cursor.row == line_info.start.row
                && cursor.column >= line_info.start.column
                && (cursor.column < line_info.end
                    || (cursor.column == line_info.end && line_info.wrap_end))
            {
                relative_cursor.column -= line_info.start.column;
                break;
            }
            cursor_idx += 1;
        }

        if cursor < scroll {
            range_start = cursor_idx.saturating_sub(3);
            range_end = range_start + visible_lines;
        } else if cursor.row >= scroll.row + visible_lines {
            range_end = cursor_idx + 3;
            range_start = range_end.saturating_sub(visible_lines);
        } else {
            range_start = 0;
            range_end = visible_lines;
            if cursor_idx >= visible_lines {
                range_end = cursor_idx + 3;
                range_start = range_end.saturating_sub(visible_lines);
            }
        }

        range_end = gutter_info.len().min(range_end);
        relative_cursor.row = cursor_idx - range_start;

        scroll.row = gutter_info[range_start].start.row;
        scroll.column = gutter_info[range_start].start.column;

        // Highlight
        let content = self.get_content("\n".into());
        let highlights = self
            .highlighter
            .highlight(&self.language_config, content.as_bytes(), None, |_| None)
            .unwrap();

        start_byte = gutter_info.first().unwrap().start_byte;
        println!("Gutter Info: {:#?}", gutter_info);
        let mut gutter_idx = 0;
        let mut highlighted_line = vec![];
        for event in highlights {
            match event.unwrap() {
                HighlightEvent::Source { start, end } => {
                    println!(
                        "Highlighting {} {} of type {:#?}",
                        start, end, highlight_type
                    );
                    if end >= gutter_info.first().unwrap().start_byte
                        && start < gutter_info.last().unwrap().end_byte
                    {
                        if start_byte < start {
                            let gutter_line = gutter_info.first().unwrap();
                            println!("Start byte {} < start {}", start_byte, start);
                            highlighted_line.push((
                                self.lines.get(gutter_line.start.row).unwrap()[start_byte..start]
                                    .to_string(),
                                highlight_type,
                            ));
                            start_byte = start;
                        }

                        while start_byte < end {
                            let gutter_line = gutter_info.get(gutter_idx).unwrap();
                            if end >= gutter_line.end_byte {
                                println!("New line {} - {}", start_byte, gutter_line.end_byte);
                                highlighted_line.push((
                                    self.lines.get(gutter_line.start.row).unwrap()
                                        [start_byte - gutter_line.start_byte..]
                                        .to_string(),
                                    highlight_type,
                                ));
                                start_byte = gutter_line.end_byte;
                                gutter_idx += 1;
                                lines.push(highlighted_line);
                                highlighted_line = vec![];
                            } else {
                                println!(
                                    "Append to current line till end {} - {}",
                                    start_byte, end
                                );
                                highlighted_line.push((
                                    self.lines.get(gutter_line.start.row).unwrap()[start_byte
                                        - gutter_line.start_byte
                                        ..end - gutter_line.start_byte]
                                        .to_string(),
                                    highlight_type,
                                ));
                                start_byte = end;
                            }
                        }
                    }
                }
                HighlightEvent::HighlightStart(s) => {
                    highlight_type = Some(s.0);
                }
                HighlightEvent::HighlightEnd => {
                    highlight_type = None;
                }
            }
        }

        (
            lines
                .get(range_start..range_end)
                .unwrap_or(&lines[range_start..])
                .to_vec(),
            relative_cursor,
            gutter_info
                .get(range_start..range_end)
                .unwrap_or(&gutter_info[range_start..])
                .to_vec(),
        )
    }

    /// Get line length
    pub fn get_line_length(&self, row: usize) -> usize {
        self.lines[row].len()
    }

    /// Get number of lines
    pub fn get_num_lines(&self) -> usize {
        self.lines.len()
    }

    /// Move cursor right in insert mode
    pub fn move_cursor_right(&self, cursor: &mut Cursor) {
        let line_length = self.get_line_length(cursor.row);
        if cursor.column == line_length {
            if cursor.row != self.get_num_lines() - 1 {
                cursor.column = 0;
                cursor.row += 1;
            }
        } else {
            cursor.column += 1;
        }
    }

    /// Move cursor left in insert mode
    pub fn move_cursor_left(&self, cursor: &mut Cursor) {
        if cursor.column == 0 {
            if cursor.row != 0 {
                cursor.row -= 1;
                cursor.column = self.get_line_length(cursor.row);
            }
        } else {
            cursor.column -= 1;
        }
    }

    /// Move cursor up in insert mode
    pub fn move_cursor_up(&self, cursor: &mut Cursor, column_level: usize) -> usize {
        if cursor.row == 0 {
            cursor.column = 0;
            cursor.column
        } else {
            cursor.row -= 1;
            if cursor.column > self.get_line_length(cursor.row) {
                cursor.column = self.get_line_length(cursor.row);
            } else {
                cursor.column = max(
                    min(column_level, self.get_line_length(cursor.row)),
                    cursor.column,
                )
            }
            column_level
        }
    }

    /// Move cursor down in insert mode
    pub fn move_cursor_down(&self, cursor: &mut Cursor, column_level: usize) -> usize {
        if cursor.row == self.get_num_lines() - 1 {
            cursor.column = self.get_line_length(cursor.row);
            cursor.column
        } else {
            cursor.row += 1;
            if cursor.column > self.get_line_length(cursor.row) {
                cursor.column = self.get_line_length(cursor.row);
            } else {
                cursor.column = max(
                    min(column_level, self.get_line_length(cursor.row)),
                    cursor.column,
                )
            }
            column_level
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::instance::Cursor;

    use super::LineBuffer;

    #[test]
    fn line_buffer_empty() {
        let buf = LineBuffer::new("".into(), None);
        assert_eq!(buf.file_path, None);
        assert_eq!(buf.lines, vec![""])
    }

    #[test]
    fn line_buffer_with_line_ending() {
        let buf = LineBuffer::new("\n".into(), None);
        assert_eq!(buf.file_path, None);
        assert_eq!(buf.lines, vec![""])
    }

    #[test]
    fn line_buffer_with_no_extra_line() {
        let buf = LineBuffer::new("Hello\nWorld".into(), None);
        assert_eq!(buf.file_path, None);
        assert_eq!(buf.lines, vec!["Hello", "World", "",])
    }

    #[test]
    fn line_buffer_with_extra_line() {
        let buf = LineBuffer::new("Hello\nWorld\n".into(), None);
        assert_eq!(buf.file_path, None);
        assert_eq!(buf.lines, vec!["Hello", "World", "",])
    }

    #[test]
    fn line_buffer_hard_wrap() {
        let mut buf = LineBuffer::new("HelloWorld".into(), None);
        let mut scroll = Cursor { row: 0, column: 0 };
        let cursor = Cursor { row: 0, column: 0 };
        let (_lines, visible_cursor, _gutter_info) =
            buf.get_visible_lines(&mut scroll, &cursor, 10, 5);
        // assert_eq!(vec!["Hello", "World", ""], lines);
        assert_eq!(visible_cursor, Cursor { row: 0, column: 0 });
    }

    #[test]
    fn move_cursor_right_same_line() {
        let buf = LineBuffer::new("Hello\nWorld\n".into(), None);
        let mut cursor = Cursor { row: 0, column: 0 };
        buf.move_cursor_right(&mut cursor);
        assert_eq!(cursor, Cursor { row: 0, column: 1 });
    }

    #[test]
    fn move_cursor_right_next_line() {
        let buf = LineBuffer::new("Hello\nWorld\n".into(), None);
        let mut cursor = Cursor { row: 0, column: 5 };
        buf.move_cursor_right(&mut cursor);
        assert_eq!(cursor, Cursor { row: 1, column: 0 });
    }

    #[test]
    fn move_cursor_right_final_line() {
        let buf = LineBuffer::new("Hello\nWorld\n".into(), None);
        let mut cursor = Cursor { row: 2, column: 0 };
        buf.move_cursor_right(&mut cursor);
        assert_eq!(cursor, Cursor { row: 2, column: 0 })
    }
}
