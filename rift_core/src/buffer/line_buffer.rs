use std::cmp::{max, min};

use super::instance::{Cursor, GutterInfo};

/// Text buffer implementation as a list of lines
#[derive(Debug)]
pub struct LineBuffer {
    pub file_path: Option<String>,
    pub lines: Vec<String>,
}

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

        Self { file_path, lines }
    }

    /// Get text buffer content as a string
    /// with the desired EOL sequence
    pub fn get_content(&self, eol_sequence: String) -> String {
        self.lines.join(&eol_sequence)
    }

    pub fn get_visible_lines(
        &self,
        scroll: &mut Cursor,
        cursor: &Cursor,
        visible_lines: usize,
        max_characters: usize,
    ) -> (Vec<String>, Cursor, Vec<GutterInfo>) {
        let mut lines = vec![];
        let mut gutter_info = vec![];
        let mut relative_cursor = Cursor {
            row: 0,
            column: cursor.column,
        };
        let mut range_start = scroll.row;
        let mut range_end = range_start + visible_lines + 3;
        let mut start = 0;
        let mut cursor_idx = 0;

        if cursor < scroll {
            range_start = cursor.row.saturating_sub(3);
            range_end = range_start + visible_lines;
        } else if cursor.row >= scroll.row + visible_lines {
            range_end = cursor.row + 3;
            range_start = range_end.saturating_sub(visible_lines);
        }

        for (line_idx, line) in self
            .lines
            .get(range_start..range_end)
            .unwrap_or(&self.lines[range_start..])
            .iter()
            .enumerate()
        {
            while start < line.len() {
                let end = (start + max_characters).min(line.len());
                lines.push(line[start..end].to_string());
                gutter_info.push(GutterInfo {
                    start: Cursor {
                        row: range_start + line_idx,
                        column: start,
                    },
                    end,
                    wrapped: start != 0,
                });

                if cursor.row == range_start + line_idx
                    && cursor.column >= start
                    && cursor.column < end
                {
                    cursor_idx = line_idx;
                    relative_cursor.column -= start;
                }

                start = end;
            }

            if line.is_empty() {
                lines.push("".to_string());
                gutter_info.push(GutterInfo {
                    start: Cursor {
                        row: range_start + line_idx,
                        column: 0,
                    },
                    end: 0,
                    wrapped: false,
                });

                if cursor.row == range_start + line_idx {
                    cursor_idx = line_idx;
                }
            }

            start = 0;
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
            if cursor_idx > visible_lines {
                range_end = cursor_idx + 3;
                range_start = range_end.saturating_sub(visible_lines);
            }
        }

        range_end = lines.len().min(range_end);
        relative_cursor.row = cursor_idx - range_start;

        scroll.row = gutter_info[range_start].start.row;
        scroll.column = gutter_info[range_start].start.column;

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
        let buf = LineBuffer::new("HelloWorld".into(), None);
        let mut scroll = Cursor { row: 0, column: 0 };
        let cursor = Cursor { row: 0, column: 0 };
        let (lines, visible_cursor, _gutter_info) =
            buf.get_visible_lines(&mut scroll, &cursor, 10, 5);
        assert_eq!(vec!["Hello", "World", ""], lines);
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
