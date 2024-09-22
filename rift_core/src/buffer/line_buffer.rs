use std::cmp::{max, min};

use super::instance::{self, Cursor};

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

    /// Get visible lines with line wrap
    pub fn get_visible_lines_with_wrap(
        &self,
        mut scroll: &mut Cursor,
        cursor: &Cursor,
        visible_lines: usize,
        max_characters: usize,
        _soft_wrap: bool,
    ) -> (Vec<String>, Cursor) {
        let mut lines = vec![];
        let mut start = scroll.column;
        let mut last_visible_row = scroll.row;
        let mut last_visible_column = 0;
        let mut visible_cursor = instance::Cursor {
            row: 0,
            column: cursor.column,
        };

        for (line_idx, line) in self
            .lines
            .get(scroll.row..scroll.row + visible_lines)
            .unwrap_or(&self.lines[scroll.row..])
            .iter()
            .enumerate()
        {
            while start < line.len() && lines.len() < visible_lines {
                let end = std::cmp::min(start + max_characters, line.len());
                lines.push(line[start..end].to_string());

                last_visible_row = scroll.row + line_idx;
                last_visible_column = end;

                if last_visible_row == cursor.row {
                    visible_cursor.row += 1;
                    if cursor.column < end {
                        visible_cursor.column = cursor.column - start;
                    }
                }

                start = end;
            }
            if line.len() == 0 && lines.len() < visible_lines {
                lines.push("".to_string());

                last_visible_row = scroll.row + line_idx;
                last_visible_column = 0;
            }
            start = 0;
        }

        if cursor.row > last_visible_row
            || (cursor.row == last_visible_row && cursor.column > last_visible_column)
        {
            scroll.row = last_visible_row;
            scroll.column = last_visible_column;
            return self.get_visible_lines_with_wrap(
                &mut scroll,
                &cursor,
                visible_lines,
                max_characters,
                _soft_wrap,
            );
        }

        (lines, visible_cursor)
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
    pub fn move_cursor_right(&self, cursor: &mut instance::Cursor) {
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
    pub fn move_cursor_left(&self, cursor: &mut instance::Cursor) {
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
    pub fn move_cursor_up(&self, cursor: &mut instance::Cursor, column_level: usize) -> usize {
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
    pub fn move_cursor_down(&self, cursor: &mut instance::Cursor, column_level: usize) -> usize {
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
    use crate::buffer::instance;

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
        let mut scroll = instance::Cursor { row: 0, column: 0 };
        let cursor = instance::Cursor { row: 0, column: 0 };
        let (lines, visible_cursor) =
            buf.get_visible_lines_with_wrap(&mut scroll, &cursor, 10, 5, false);
        assert_eq!(vec!["Hello", "World", ""], lines);
        assert_eq!(visible_cursor, instance::Cursor { row: 0, column: 0 });
    }

    #[test]
    fn move_cursor_right_same_line() {
        let buf = LineBuffer::new("Hello\nWorld\n".into(), None);
        let mut cursor = instance::Cursor { row: 0, column: 0 };
        buf.move_cursor_right(&mut cursor);
        assert_eq!(cursor, instance::Cursor { row: 0, column: 1 });
    }

    #[test]
    fn move_cursor_right_next_line() {
        let buf = LineBuffer::new("Hello\nWorld\n".into(), None);
        let mut cursor = instance::Cursor { row: 0, column: 5 };
        buf.move_cursor_right(&mut cursor);
        assert_eq!(cursor, instance::Cursor { row: 1, column: 0 });
    }

    #[test]
    fn move_cursor_right_final_line() {
        let buf = LineBuffer::new("Hello\nWorld\n".into(), None);
        let mut cursor = instance::Cursor { row: 2, column: 0 };
        buf.move_cursor_right(&mut cursor);
        assert_eq!(cursor, instance::Cursor { row: 2, column: 0 })
    }
}
