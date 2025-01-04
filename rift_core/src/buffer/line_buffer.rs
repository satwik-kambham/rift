use std::{
    cmp::{max, min},
    collections::{HashMap, VecDeque},
};

use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use crate::lsp::client::LSPClientHandle;

use super::instance::{Cursor, Edit, GutterInfo, HighlightType, Selection};

/// Text buffer implementation as a list of lines
pub struct LineBuffer {
    pub file_path: Option<String>,
    pub lines: Vec<String>,
    highlighter: Highlighter,
    language_config: HighlightConfiguration,
    highlight_map: HashMap<String, HighlightType>,
    highlight_names: Vec<String>,
    pub modified: bool,
    pub changes: VecDeque<Edit>,
    pub change_idx: usize,
    pub version: usize,
}

pub type HighlightedText = Vec<Vec<(String, HighlightType, bool)>>;

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
        let highlight_map: HashMap<String, HighlightType> = HashMap::from([
            ("attribute".into(), HighlightType::Red),
            ("constant".into(), HighlightType::Red),
            ("function.builtin".into(), HighlightType::Purple),
            ("function".into(), HighlightType::Blue),
            ("keyword".into(), HighlightType::Purple),
            ("operator".into(), HighlightType::Purple),
            ("property".into(), HighlightType::Yellow),
            ("punctuation".into(), HighlightType::White),
            ("punctuation.bracket".into(), HighlightType::Orange),
            ("punctuation.delimiter".into(), HighlightType::Orange),
            ("string".into(), HighlightType::Green),
            ("string.special".into(), HighlightType::Orange),
            ("comment".into(), HighlightType::Gray),
            ("comment.documentation".into(), HighlightType::Gray),
            ("tag".into(), HighlightType::Turquoise),
            ("type".into(), HighlightType::Yellow),
            ("type.builtin".into(), HighlightType::Yellow),
            ("variable".into(), HighlightType::Red),
            ("variable.builtin".into(), HighlightType::Orange),
            ("variable.parameter".into(), HighlightType::Red),
        ]);
        let mut language_config = HighlightConfiguration::new(
            tree_sitter_rust::LANGUAGE.into(),
            "rust",
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            "",
        )
        .unwrap();
        let highlight_names: Vec<String> =
            highlight_map.keys().map(|key| key.to_string()).collect();
        language_config.configure(&highlight_names);
        tracing::info!("Highlight Names: {:#?}", language_config.names());

        Self {
            file_path,
            lines,
            highlighter,
            language_config,
            highlight_map,
            highlight_names,
            modified: false,
            changes: VecDeque::new(),
            change_idx: 0,
            version: 1,
        }
    }

    /// Get text buffer content as a string
    /// with the desired EOL sequence
    pub fn get_content(&self, eol_sequence: String) -> String {
        self.lines.join(&eol_sequence)
    }

    /// Get selection as string
    pub fn get_selection(&self, selection: &Selection) -> String {
        let mut content = String::new();
        let (start, end) = selection.in_order();
        let mut cursor = *start;

        while &cursor < end {
            if cursor.row == end.row {
                content.push_str(&self.lines[cursor.row][cursor.column..end.column]);
            } else {
                content.push_str(&self.lines[cursor.row][cursor.column..]);
            }
            content.push('\n');
            cursor.row += 1;
            cursor.column = 0;
        }
        content
    }

    pub fn get_visible_lines(
        &mut self,
        scroll: &mut Cursor,
        selection: &Selection,
        visible_lines: usize,
        max_characters: usize,
        eol_sequence: String,
    ) -> (HighlightedText, Cursor, Vec<GutterInfo>) {
        let mut lines = vec![];
        let mut gutter_info = vec![];
        let mut relative_cursor = Cursor {
            row: 0,
            column: selection.cursor.column,
        };
        let mut range_start = scroll.row;
        let mut range_end = range_start + visible_lines + 3;
        let mut start = 0;
        let mut cursor_idx: usize = 0;
        let mut start_byte = 0;
        let mut highlight_type = HighlightType::None;

        // Calculate range
        if &selection.cursor < scroll {
            range_start = selection.cursor.row.saturating_sub(3);
            range_end = range_start + visible_lines;
        } else if selection.cursor.row >= scroll.row + visible_lines {
            range_end = selection.cursor.row + 3;
            range_start = range_end.saturating_sub(visible_lines);
        }

        // Calculate start byte
        for line in self.lines.get(..range_start).unwrap() {
            start_byte += line.len() + eol_sequence.len();
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
                let eol_len = if end == line.len() {
                    eol_sequence.len()
                } else {
                    0
                };
                let end_byte = start_byte + end - start + eol_len;
                gutter_info.push(GutterInfo {
                    start: Cursor {
                        row: range_start + line_idx,
                        column: start,
                    },
                    end,
                    wrapped: start != 0,
                    wrap_end: end == line.len(),
                    start_byte,
                    end_byte,
                });

                start_byte = end_byte;
                start = end;
            }

            if line.is_empty() {
                let end_byte = start_byte + eol_sequence.len();
                gutter_info.push(GutterInfo {
                    start: Cursor {
                        row: range_start + line_idx,
                        column: 0,
                    },
                    end: 0,
                    wrapped: false,
                    wrap_end: true,
                    start_byte,
                    end_byte,
                });
                start_byte = end_byte;
            }

            start = 0;
        }

        // Calculate relative cursor position
        for line_info in &gutter_info {
            if selection.cursor.row == line_info.start.row
                && selection.cursor.column >= line_info.start.column
                && (selection.cursor.column < line_info.end
                    || (selection.cursor.column == line_info.end && line_info.wrap_end))
            {
                relative_cursor.column -= line_info.start.column;
                break;
            }
            cursor_idx += 1;
        }

        if &selection.cursor < scroll {
            range_start = cursor_idx.saturating_sub(3);
            range_end = range_start + visible_lines;
        } else if selection.cursor.row >= scroll.row + visible_lines {
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
        let mut gutter_idx = 0;
        let mut highlighted_line = vec![];
        let selection_start = if selection.mark > selection.cursor {
            selection.cursor
        } else {
            selection.mark
        };
        let selection_end = if selection.mark > selection.cursor {
            selection.mark
        } else {
            selection.cursor
        };
        for event in highlights {
            match event.unwrap() {
                HighlightEvent::Source { start, end } => {
                    if end >= gutter_info.first().unwrap().start_byte
                        && start < gutter_info.last().unwrap().end_byte
                    {
                        // Append text present before syntax highlighting range
                        if start_byte < start {
                            let gutter_line = gutter_info.first().unwrap();
                            let line = self.lines.get(gutter_line.start.row).unwrap()
                                [..start - start_byte]
                                .to_string();
                            let segments = LineBuffer::split_line(
                                &line,
                                &gutter_line.start,
                                &Cursor {
                                    row: gutter_line.start.row,
                                    column: start - start_byte,
                                },
                                &selection_start,
                                &selection_end,
                            );
                            for (segment, selected) in segments {
                                highlighted_line.push((segment, highlight_type, selected))
                            }
                            start_byte = start;
                        }

                        while start_byte < end && gutter_idx < gutter_info.len() {
                            let gutter_line = gutter_info.get(gutter_idx).unwrap();
                            // If highlight range is outside a single visual line then
                            // highlight till line end and go to next line
                            if end >= gutter_line.end_byte {
                                let line = self.lines.get(gutter_line.start.row).unwrap()
                                    [gutter_line.start.column + start_byte - gutter_line.start_byte
                                        ..gutter_line.end]
                                    .to_string();
                                let segments = LineBuffer::split_line(
                                    &line,
                                    &Cursor {
                                        row: gutter_line.start.row,
                                        column: gutter_line.start.column + start_byte
                                            - gutter_line.start_byte,
                                    },
                                    &Cursor {
                                        row: gutter_line.start.row,
                                        column: gutter_line.end,
                                    },
                                    &selection_start,
                                    &selection_end,
                                );
                                for (segment, selected) in segments {
                                    highlighted_line.push((segment, highlight_type, selected))
                                }
                                start_byte = gutter_line.end_byte;
                                gutter_idx += 1;
                                lines.push(highlighted_line);
                                highlighted_line = vec![];
                            } else {
                                // If not append till end of highlight
                                let line = self.lines.get(gutter_line.start.row).unwrap()
                                    [gutter_line.start.column + start_byte - gutter_line.start_byte
                                        ..gutter_line.start.column + end - gutter_line.start_byte]
                                    .to_string();
                                let segments = LineBuffer::split_line(
                                    &line,
                                    &Cursor {
                                        row: gutter_line.start.row,
                                        column: gutter_line.start.column + start_byte
                                            - gutter_line.start_byte,
                                    },
                                    &Cursor {
                                        row: gutter_line.start.row,
                                        column: gutter_line.start.column + end
                                            - gutter_line.start_byte,
                                    },
                                    &selection_start,
                                    &selection_end,
                                );
                                for (segment, selected) in segments {
                                    highlighted_line.push((segment, highlight_type, selected))
                                }
                                start_byte = end;
                            }
                        }
                    }
                }
                HighlightEvent::HighlightStart(s) => {
                    highlight_type = self.highlight_map[&self.highlight_names[s.0]];
                }
                HighlightEvent::HighlightEnd => {
                    highlight_type = HighlightType::None;
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

    /// Splits line based on selection
    fn split_line(
        line: &String,
        line_start: &Cursor,
        line_end: &Cursor,
        selection_start: &Cursor,
        selection_end: &Cursor,
    ) -> Vec<(String, bool)> {
        let mut segments = vec![];

        // If selection overlaps with line
        if selection_end > line_start && selection_start < line_end {
            // If selection completely overlaps
            if selection_start <= line_start && selection_end >= line_end {
                segments.push((line.to_string(), true));
            } else if selection_start >= line_start && selection_end <= line_end {
                // Selection completely withing line
                let (first, middle) = line.split_at(selection_start.column - line_start.column);
                let (middle, last) = middle.split_at(selection_end.column - selection_start.column);

                segments.push((first.to_string(), false));
                segments.push((middle.to_string(), true));
                segments.push((last.to_string(), false));
            } else if selection_start >= line_start {
                // Selection on right portion of line
                let (first, last) = line.split_at(selection_start.column - line_start.column);

                segments.push((first.to_string(), false));
                segments.push((last.to_string(), true));
            } else if selection_end <= line_end {
                // Selection on left portion of line
                let (first, last) = line.split_at(selection_end.column - line_start.column);

                segments.push((first.to_string(), true));
                segments.push((last.to_string(), false));
            }
        } else {
            segments.push((line.to_string(), false));
        }

        segments
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

    /// Move cursor to start of line
    pub fn move_cursor_line_start(&self, cursor: &mut Cursor) {
        cursor.column = 0;
    }

    /// Move cursor to end of line
    pub fn move_cursor_line_end(&self, cursor: &mut Cursor) {
        cursor.column = self.get_line_length(cursor.row);
    }

    /// Move cursor to start of buffer
    pub fn move_cursor_buffer_start(&self, cursor: &mut Cursor) {
        cursor.row = 0;
        cursor.column = 0;
    }

    /// Move cursor to end of buffer
    pub fn move_cursor_buffer_end(&self, cursor: &mut Cursor) {
        cursor.row = self.get_num_lines() - 1;
        cursor.column = self.get_line_length(cursor.row);
    }

    /// Insert text at cursor position and return update cursor position
    pub fn insert_text_no_log(&mut self, text: &str, cursor: &Cursor) -> Cursor {
        self.modified = true;

        let mut updated_cursor = *cursor;
        let current_line = self.lines[cursor.row].clone();
        let mut text_iter = text.split('\n');
        let (s1, s2) = current_line.split_at(cursor.column);
        let mut s1 = s1.to_string();
        let e = text_iter.next().unwrap();
        updated_cursor.column += e.len();
        s1.push_str(e);
        self.lines[cursor.row] = s1;
        for i in text_iter {
            updated_cursor.row += 1;
            updated_cursor.column = i.len();
            self.lines.insert(updated_cursor.row, i.to_owned());
        }
        let mut current_line = self.lines[updated_cursor.row].clone();
        current_line.push_str(s2);
        self.lines[updated_cursor.row] = current_line;

        updated_cursor
    }

    pub fn insert_text(
        &mut self,
        text: &str,
        cursor: &Cursor,
        lsp_handle: &LSPClientHandle,
        log: bool,
    ) -> Cursor {
        let updated_cursor = self.insert_text_no_log(text, cursor);

        if log {
            self.changes.truncate(self.change_idx);
            self.changes.push_back(Edit::Insert {
                start: *cursor,
                end: updated_cursor,
                text: text.to_owned(),
            });
            self.change_idx = self.changes.len();
        }
        self.version += 1;

        lsp_handle
            .send_notification_sync(
                "textDocument/didChange".to_string(),
                Some(LSPClientHandle::did_change_text_document(
                    self.file_path.clone().unwrap(),
                    self.version,
                    // Selection {
                    //     cursor: *cursor,
                    //     mark: *cursor,
                    // },
                    // text.to_string(),
                    self.get_content("\n".to_owned()),
                )),
            )
            .unwrap();

        updated_cursor
    }

    /// Removes the selected text and returns the updated cursor position
    /// and the deleted text
    pub fn remove_text_no_log(&mut self, selection: &Selection) -> (String, Cursor) {
        self.modified = true;

        let start = if selection.mark < selection.cursor {
            selection.mark
        } else {
            selection.cursor
        };
        let end = if selection.mark < selection.cursor {
            selection.cursor
        } else {
            selection.mark
        };

        if start.row == end.row {
            let current_line = self.lines[start.row].clone();
            let (first, second) = current_line.split_at(end.column);
            let (first, middle) = first.split_at(start.column);
            self.lines[start.row] = first.to_owned() + second;

            (middle.to_owned(), start)
        } else {
            let mut buf = String::new();

            let current_line = self.lines[end.row].clone();
            let (first, second) = current_line.split_at(end.column);
            buf.insert_str(0, first);
            self.lines.remove(end.row);

            for i in (start.row + 1..end.row).rev() {
                let current_line = self.lines.remove(i);
                buf.insert(0, '\n');
                buf.insert_str(0, &current_line);
            }

            let current_line = self.lines[start.row].clone();
            let (first, middle) = current_line.split_at(start.column);
            buf.insert(0, '\n');
            buf.insert_str(0, middle);
            self.lines[start.row] = first.to_owned() + second;

            (buf, start)
        }
    }

    pub fn remove_text(
        &mut self,
        selection: &Selection,
        lsp_handle: &LSPClientHandle,
        log: bool,
    ) -> (String, Cursor) {
        let (text, cursor) = self.remove_text_no_log(selection);

        let (start, end) = selection.in_order();
        if log {
            self.changes.truncate(self.change_idx);
            self.changes.push_back(Edit::Delete {
                start: *start,
                end: *end,
                text: text.to_owned(),
            });
            self.change_idx = self.changes.len();
        }
        self.version += 1;

        lsp_handle
            .send_notification_sync(
                "textDocument/didChange".to_string(),
                Some(LSPClientHandle::did_change_text_document(
                    self.file_path.clone().unwrap(),
                    self.version,
                    // *selection,
                    // "".to_string(),
                    self.get_content("\n".to_owned()),
                )),
            )
            .unwrap();

        (text, cursor)
    }

    /// Undo
    pub fn undo(&mut self, lsp_handle: &LSPClientHandle) -> Option<Cursor> {
        self.version += 1;
        if self.change_idx > 0 {
            self.change_idx -= 1;
            let edit = self.changes.get(self.change_idx).unwrap();
            match edit {
                Edit::Insert {
                    start,
                    end,
                    text: _,
                } => {
                    let (_text, cursor) = self.remove_text(
                        &Selection {
                            cursor: *start,
                            mark: *end,
                        },
                        lsp_handle,
                        false,
                    );
                    return Some(cursor);
                }
                Edit::Delete {
                    start,
                    end: _,
                    text,
                } => {
                    let cursor = self.insert_text(&text.clone(), &start.clone(), lsp_handle, false);
                    return Some(cursor);
                }
            }
        }
        None
    }

    /// Redo
    pub fn redo(&mut self, lsp_handle: &LSPClientHandle) -> Option<Cursor> {
        self.version += 1;
        if self.change_idx < self.changes.len() {
            self.change_idx += 1;
            let edit = self.changes.get(self.change_idx - 1).unwrap();
            match edit {
                Edit::Insert {
                    start,
                    end: _,
                    text,
                } => {
                    let cursor = self.insert_text(&text.clone(), &start.clone(), lsp_handle, false);
                    return Some(cursor);
                }
                Edit::Delete {
                    start,
                    end,
                    text: _,
                } => {
                    let (_text, cursor) = self.remove_text(
                        &Selection {
                            cursor: *start,
                            mark: *end,
                        },
                        lsp_handle,
                        false,
                    );
                    return Some(cursor);
                }
            }
        }
        None
    }

    /// Get indentation level (number of spaces) of given row
    pub fn get_indentation_level(&self, row: usize) -> usize {
        let line = &self.lines[row];
        line.chars().take_while(|c| *c == ' ').count()
    }

    /// Add indentation to the selected lines and returns the updated cursor position
    pub fn add_indentation(
        &mut self,
        selection: &Selection,
        tab_size: usize,
        lsp_handle: &LSPClientHandle,
    ) -> Selection {
        self.modified = true;

        let mut updated_selection = *selection;
        let tab = " ".repeat(tab_size);
        updated_selection.mark.column += tab_size;
        updated_selection.cursor.column += tab_size;
        let (start, end) = selection.in_order();
        for i in start.row..=end.row {
            self.insert_text(&tab, &Cursor { row: i, column: 0 }, lsp_handle, true);
        }
        updated_selection
    }

    /// Remove indentation from the selected lines if present and returns the updated cursor position
    pub fn remove_indentation(
        &mut self,
        selection: &Selection,
        tab_size: usize,
        lsp_handle: &LSPClientHandle,
    ) -> Selection {
        self.modified = true;

        let mut updated_selection = *selection;
        let tab = " ".repeat(tab_size);
        let (start, end) = selection.in_order();
        let (start_new, end_new) = updated_selection.in_order_mut();
        for i in start.row..=end.row {
            let current_line = &self.lines[i];
            if current_line.starts_with(&tab) {
                self.remove_text(
                    &Selection {
                        cursor: Cursor { row: i, column: 0 },
                        mark: Cursor {
                            row: i,
                            column: tab_size,
                        },
                    },
                    lsp_handle,
                    true,
                );

                if i == start.row {
                    start_new.column = start_new.column.saturating_sub(tab_size);
                }
                if i == end.row {
                    end_new.column = end_new.column.saturating_sub(tab_size);
                }
            }
        }
        updated_selection
    }

    /// Adds line to selection and returns updated selection
    pub fn select_line(&self, selection: &Selection) -> Selection {
        let mut updated_selection = *selection;
        if selection.mark.column == 0
            && selection.cursor.column == self.get_line_length(selection.cursor.row)
        {
            updated_selection.cursor.row += 1;
        }
        updated_selection.mark.column = 0;
        updated_selection.cursor.column = self.get_line_length(updated_selection.cursor.row);
        updated_selection
    }

    /// Adds word to selection and returns updated selection
    pub fn select_word(&self, selection: &Selection) -> Selection {
        let mut updated_selection = *selection;
        let line = &self.lines[selection.cursor.row];
        // let mut start = selection.cursor.column;
        let mut end = selection.cursor.column;
        // while start > 0 && line.chars().nth(start - 1).unwrap().is_alphanumeric() {
        //     start -= 1;
        // }
        while end < line.len() && line.chars().nth(end).unwrap().is_alphanumeric() {
            end += 1;
        }
        // updated_selection.mark.column = start;
        updated_selection.cursor.column = end;
        updated_selection
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::instance::{Cursor, Selection};

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
        let selection = Selection {
            mark: Cursor { row: 0, column: 0 },
            cursor: Cursor { row: 0, column: 0 },
        };
        let (_lines, visible_cursor, _gutter_info) =
            buf.get_visible_lines(&mut scroll, &selection, 10, 5, "\n".into());
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
