use std::{
    cmp::{max, min},
    collections::{HashSet, VecDeque},
};

use crate::lsp::client::LSPClientHandle;

use super::highlight::{TreeSitterParams, build_highlight_params, detect_language};
use super::instance::{
    Attribute, Cursor, Edit, GutterInfo, HighlightType, Language, Range, Selection,
};

use ropey::Rope;
use tree_sitter_highlight::{HighlightEvent, Highlighter};

pub struct RopeBuffer {
    file_path: Option<String>,
    pub display_name: Option<String>,
    pub special: bool,
    buffer: Rope,
    pub modified: bool,
    changes: VecDeque<Edit>,
    change_idx: usize,
    pub version: usize,
    pub language: Language,
    highlighter: Highlighter,
    highlight_params: Option<TreeSitterParams>,
    pub input: String,
    pub input_hook: Option<String>,
}

pub type HighlightedText = Vec<Vec<(String, HashSet<Attribute>)>>;
pub struct VisibleLineParams {
    pub visible_lines: usize,
    pub max_characters: usize,
    pub eol_sequence: String,
}

impl RopeBuffer {
    /// Create a rope buffer
    pub fn new(
        initial_text: String,
        file_path: Option<String>,
        workspace_folder: &str,
        special: bool,
    ) -> Self {
        let buffer = Rope::from_str(&initial_text);

        let language = detect_language(&file_path);
        // Syntax highlighter
        let highlighter = Highlighter::new();
        let highlight_params = build_highlight_params(language);

        let mut buffer = Self {
            file_path: None,
            display_name: None,
            special,
            buffer,
            highlighter,
            highlight_params,
            modified: false,
            changes: VecDeque::new(),
            change_idx: 0,
            version: 1,
            language,
            input: String::new(),
            input_hook: None,
        };

        buffer.set_file_path(file_path, workspace_folder);
        buffer
    }

    pub fn file_path(&self) -> Option<&String> {
        self.file_path.as_ref()
    }

    pub fn set_file_path(&mut self, file_path: Option<String>, workspace_folder: &str) {
        self.file_path = file_path.clone();

        self.display_name = file_path.as_ref().map(|path| {
            let path = std::path::Path::new(path);
            let workspace = std::path::Path::new(workspace_folder);
            let relative_path = path.strip_prefix(workspace).unwrap_or(path);
            relative_path.to_string_lossy().to_string()
        });

        let language = detect_language(&self.file_path);
        if self.language != language {
            self.language = language;
            self.highlight_params = build_highlight_params(language);
        }
    }

    /// Get text buffer content as a string
    /// with the desired EOL sequence
    pub fn get_content(&self, _eol_sequence: String) -> String {
        self.buffer.to_string()
    }

    pub fn set_content(&mut self, content: String) {
        self.buffer = Rope::from_str(&content);
    }

    /// Get a portion text buffer content as a string
    pub fn get_content_range(&self, start_line: usize, end_line: usize) -> String {
        let start_idx = self.buffer.line_to_char(start_line);
        let end_idx = self.buffer.line_to_char(end_line);
        self.buffer.slice(start_idx..end_idx).to_string()
    }

    /// Get selection as string
    pub fn get_selection(&self, selection: &Selection) -> String {
        let (start, end) = selection.in_order();
        let start_idx = self.buffer.line_to_char(start.row) + start.column;
        let end_idx = self.buffer.line_to_char(end.row) + end.column;
        self.buffer.slice(start_idx..end_idx).to_string()
    }

    pub fn split_ranges(ranges: Vec<Range>) -> Vec<Range> {
        let mut boundaries = vec![];
        for range in &ranges {
            boundaries.push(range.start);
            boundaries.push(range.end + 1);
        }

        boundaries.sort();
        boundaries.dedup();

        let mut result = vec![];
        for window in boundaries.windows(2) {
            let start = window[0];
            let end = window[1] - 1;

            let mut active_attributes = HashSet::new();
            for range in &ranges {
                if start <= range.end && end >= range.start {
                    active_attributes.extend(range.attributes.clone());
                }
            }

            if !active_attributes.is_empty() {
                result.push(Range {
                    start,
                    end,
                    attributes: active_attributes,
                });
            }
        }

        result
    }

    pub fn byte_index_from_cursor(&self, cursor: &Cursor) -> usize {
        self.buffer.line_to_char(cursor.row) + cursor.column
    }

    pub fn get_visible_lines(
        &mut self,
        scroll: &mut Cursor,
        cursor: &Cursor,
        selection: &Selection,
        params: &VisibleLineParams,
        mut extra_segments: Vec<Range>,
    ) -> (HighlightedText, Cursor, Vec<GutterInfo>) {
        let max_characters = params.max_characters.saturating_sub(3).max(1);
        let mut segments = vec![];
        segments.append(&mut extra_segments);

        let num_lines = self.get_num_lines();
        let mut range_start = scroll.row.min(num_lines.saturating_sub(1));
        let mut range_end = range_start + params.visible_lines + 3;

        if !self.special {
            if cursor < scroll {
                range_start = cursor.row;
                range_end = range_start + params.visible_lines;
            } else if cursor.row >= scroll.row + params.visible_lines {
                range_end = cursor.row + 1;
                range_start = range_end.saturating_sub(params.visible_lines);
            }
        }

        let mut gutter_info = vec![];
        let end_line = range_end.min(num_lines);
        for line_idx in range_start..end_line {
            let line = self.buffer.line(line_idx);
            let line_length = self.get_line_length(line_idx);
            let mut eol_len = line.len_chars().saturating_sub(line_length);
            if eol_len == 0 {
                eol_len = 1;
            }

            let line_start = self.buffer.line_to_char(line_idx);
            if line_length == 0 {
                gutter_info.push(GutterInfo {
                    start: Cursor {
                        row: line_idx,
                        column: 0,
                    },
                    end: 0,
                    wrapped: false,
                    wrap_end: true,
                    start_byte: line_start,
                    end_byte: line_start + eol_len,
                });
                continue;
            }

            let mut start = 0;
            while start < line_length {
                let end = (start + max_characters).min(line_length);
                let wrap_end = end == line_length;
                let end_byte = line_start + end + if wrap_end { eol_len } else { 0 };
                gutter_info.push(GutterInfo {
                    start: Cursor {
                        row: line_idx,
                        column: start,
                    },
                    end,
                    wrapped: start != 0,
                    wrap_end,
                    start_byte: line_start + start,
                    end_byte,
                });
                start = end;
            }
        }

        for gutter_line in &gutter_info {
            let visible_end = if gutter_line.end_byte > gutter_line.start_byte {
                gutter_line.end_byte - 1
            } else {
                gutter_line.start_byte
            };
            segments.push(Range {
                start: gutter_line.start_byte,
                end: visible_end,
                attributes: HashSet::from([Attribute::Visible]),
            });
        }

        let mut relative_cursor = Cursor {
            row: 0,
            column: cursor.column,
        };

        if !self.special {
            let mut cursor_idx: usize = 0;
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
                range_start = cursor_idx.saturating_sub(1);
                range_end = range_start + params.visible_lines;
            } else if cursor.row >= scroll.row + params.visible_lines {
                range_end = cursor_idx + 1;
                range_start = range_end.saturating_sub(params.visible_lines);
            } else {
                range_start = 0;
                range_end = params.visible_lines;
                if cursor_idx >= params.visible_lines {
                    range_end = cursor_idx + 1;
                    range_start = range_end.saturating_sub(params.visible_lines);
                }
            }

            range_end = gutter_info.len().min(range_end);
            relative_cursor.row = cursor_idx - range_start;

            if !gutter_info.is_empty() {
                scroll.row = gutter_info[range_start].start.row;
                scroll.column = gutter_info[range_start].start.column;
            }

            let (selection_start, selection_end) = selection.in_order();
            if selection_start != selection_end {
                segments.push(Range {
                    start: self.byte_index_from_cursor(selection_start),
                    end: self.byte_index_from_cursor(selection_end),
                    attributes: HashSet::from([Attribute::Select]),
                });
            }

            segments.push(Range {
                start: self.byte_index_from_cursor(cursor),
                end: self.byte_index_from_cursor(cursor),
                attributes: HashSet::from([Attribute::Cursor]),
            });
        } else {
            range_start = 0;
            range_end = params.visible_lines;
            range_end = gutter_info.len().min(range_end);
            if !gutter_info.is_empty() {
                scroll.row = gutter_info[range_start].start.row;
                scroll.column = gutter_info[range_start].start.column;
            }
        }

        if let Some(highlight_params) = &self.highlight_params {
            let mut highlight_type = HighlightType::None;

            if let (Some(first), Some(last)) = (gutter_info.first(), gutter_info.last()) {
                let start_char = self.buffer.line_to_char(first.start.row);
                let end_line_idx = (last.start.row + 1).min(self.buffer.len_lines());
                let end_char = self.buffer.line_to_char(end_line_idx);
                let content = self.buffer.slice(start_char..end_char).to_string();

                let highlights = self
                    .highlighter
                    .highlight(
                        &highlight_params.language_config,
                        content.as_bytes(),
                        None,
                        |_| None,
                    )
                    .unwrap();

                for event in highlights {
                    match event.unwrap() {
                        HighlightEvent::Source { start, end } => {
                            let start = content[..start].chars().count() + start_char;
                            let end = content[..end].chars().count() + start_char;
                            if end >= first.start_byte && start <= last.end_byte {
                                segments.push(Range {
                                    start,
                                    end: end.saturating_sub(1),
                                    attributes: HashSet::from([Attribute::Highlight(
                                        highlight_type,
                                    )]),
                                });
                            }
                        }
                        HighlightEvent::HighlightStart(s) => {
                            highlight_type = highlight_params.highlight_map
                                [&highlight_params.highlight_names[s.0]];
                        }
                        HighlightEvent::HighlightEnd => {
                            highlight_type = HighlightType::None;
                        }
                    }
                }
            }
        }

        let mut split_segments = RopeBuffer::split_ranges(segments);
        let mut split_segments_iter = split_segments.iter_mut().peekable();
        let mut lines = vec![];
        let mut highlighted_line = vec![];

        while split_segments_iter.next_if(|s| s.start < gutter_info.first().unwrap().start_byte).is_some()
        {
        }

        for line_info in &gutter_info {
            while let Some(segment) = split_segments_iter.next_if(|s| s.end < line_info.end_byte) {
                let line_end = self.get_line_length(line_info.start.row);
                let line_start = self.buffer.line_to_char(line_info.start.row);
                let seg_start_in_line =
                    line_info.start.column + segment.start.saturating_sub(line_info.start_byte);
                let mut seg_end_in_line =
                    line_info.start.column + (segment.end.saturating_sub(line_info.start_byte) + 1);
                if seg_end_in_line > line_end {
                    seg_end_in_line = line_end;
                }
                if seg_end_in_line < seg_start_in_line {
                    seg_end_in_line = seg_start_in_line;
                }

                let mut buffer_segment = self
                    .buffer
                    .slice((line_start + seg_start_in_line)..(line_start + seg_end_in_line))
                    .to_string();
                if segment.attributes.contains(&Attribute::Cursor) && buffer_segment.is_empty() {
                    buffer_segment.push(' ');
                }
                let attributes = segment.attributes.clone();
                highlighted_line.push((buffer_segment, attributes));
                if segment.end == line_info.end_byte.saturating_sub(1) {
                    lines.push(highlighted_line);
                    highlighted_line = vec![];
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
        let line = self.buffer.line(row);

        line.chars().take_while(|&c| c != '\n' && c != '\r').count()
    }

    /// Get number of lines
    pub fn get_num_lines(&self) -> usize {
        self.buffer.len_lines()
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

    /// Reset buffer
    pub fn reset(&mut self) {
        self.changes.clear();
        self.change_idx = 0;
    }

    /// Insert text at cursor position and return update cursor position
    pub fn insert_text_no_log(&mut self, text: &str, cursor: &Cursor) -> Cursor {
        self.modified = true;

        let mut updated_cursor = *cursor;
        let mut text_iter = text.split('\n');
        let current_line_part = text_iter.next().unwrap_or("");
        updated_cursor.column += current_line_part.len();
        for segment in text_iter {
            updated_cursor.row += 1;
            updated_cursor.column = segment.len();
        }

        let char_idx = self.buffer.line_to_char(cursor.row) + cursor.column;
        self.buffer.insert(char_idx, text);

        updated_cursor
    }

    pub fn insert_text(
        &mut self,
        text: &str,
        cursor: &Cursor,
        lsp_handle: &Option<&mut LSPClientHandle>,
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

        if let Some(lsp_handle) = lsp_handle {
            let sync_kind = if lsp_handle.initialize_capabilities["textDocumentSync"].is_u64() {
                lsp_handle.initialize_capabilities["textDocumentSync"]
                    .as_u64()
                    .unwrap()
            } else if lsp_handle.initialize_capabilities["textDocumentSync"]["change"].is_u64() {
                lsp_handle.initialize_capabilities["textDocumentSync"]["change"]
                    .as_u64()
                    .unwrap()
            } else {
                0
            };

            if sync_kind != 0 {
                if sync_kind == 1 {
                    lsp_handle
                        .send_notification_sync(
                            "textDocument/didChange".to_string(),
                            Some(LSPClientHandle::did_change_text_document(
                                self.file_path.clone().unwrap(),
                                self.version,
                                None,
                                self.get_content("\n".to_owned()),
                            )),
                        )
                        .unwrap();
                } else if sync_kind == 2 {
                    lsp_handle
                        .send_notification_sync(
                            "textDocument/didChange".to_string(),
                            Some(LSPClientHandle::did_change_text_document(
                                self.file_path.clone().unwrap(),
                                self.version,
                                Some(Selection {
                                    cursor: *cursor,
                                    mark: *cursor,
                                }),
                                text.to_string(),
                            )),
                        )
                        .unwrap();
                }
            }
        }

        updated_cursor
    }

    /// Removes the selected text and returns the updated cursor position
    /// and the deleted text
    pub fn remove_text_no_log(&mut self, selection: &Selection) -> (String, Cursor) {
        self.modified = true;

        let (start, end) = selection.in_order();
        let start_idx = self.buffer.line_to_char(start.row) + start.column;
        let end_idx = self.buffer.line_to_char(end.row) + end.column;

        let deleted_text = self.buffer.slice(start_idx..end_idx).to_string();
        self.buffer.remove(start_idx..end_idx);

        (deleted_text, *start)
    }

    pub fn remove_text(
        &mut self,
        selection: &Selection,
        lsp_handle: &Option<&mut LSPClientHandle>,
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

        if let Some(lsp_handle) = lsp_handle {
            let sync_kind = if lsp_handle.initialize_capabilities["textDocumentSync"].is_u64() {
                lsp_handle.initialize_capabilities["textDocumentSync"]
                    .as_u64()
                    .unwrap()
            } else if lsp_handle.initialize_capabilities["textDocumentSync"]["change"].is_u64() {
                lsp_handle.initialize_capabilities["textDocumentSync"]["change"]
                    .as_u64()
                    .unwrap()
            } else {
                0
            };

            if sync_kind != 0 {
                if sync_kind == 1 {
                    lsp_handle
                        .send_notification_sync(
                            "textDocument/didChange".to_string(),
                            Some(LSPClientHandle::did_change_text_document(
                                self.file_path.clone().unwrap(),
                                self.version,
                                None,
                                self.get_content("\n".to_owned()),
                            )),
                        )
                        .unwrap();
                } else if sync_kind == 2 {
                    lsp_handle
                        .send_notification_sync(
                            "textDocument/didChange".to_string(),
                            Some(LSPClientHandle::did_change_text_document(
                                self.file_path.clone().unwrap(),
                                self.version,
                                Some(*selection),
                                "".to_string(),
                            )),
                        )
                        .unwrap();
                }
            }
        }

        (text, cursor)
    }

    /// Undo
    pub fn undo(&mut self, lsp_handle: &Option<&mut LSPClientHandle>) -> Option<Cursor> {
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
    pub fn redo(&mut self, lsp_handle: &Option<&mut LSPClientHandle>) -> Option<Cursor> {
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
        let line = self.buffer.line(row);
        line.chars().take_while(|c| *c == ' ').count()
    }

    /// Add indentation to the selected lines and returns the updated cursor position
    pub fn add_indentation(
        &mut self,
        selection: &Selection,
        tab_size: usize,
        lsp_handle: &Option<&mut LSPClientHandle>,
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
        lsp_handle: &Option<&mut LSPClientHandle>,
    ) -> Selection {
        self.modified = true;

        let mut updated_selection = *selection;
        let tab = " ".repeat(tab_size);
        let tab_chars: Vec<char> = tab.chars().collect();
        let (start, end) = selection.in_order();
        let (start_new, end_new) = updated_selection.in_order_mut();

        for i in start.row..=end.row {
            let line_chars: Vec<char> = self
                .buffer
                .line(i)
                .chars()
                .take(self.get_line_length(i))
                .collect();

            if line_chars.len() >= tab_size && line_chars.iter().take(tab_size).eq(tab_chars.iter())
            {
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

    /// Comment/Uncomment the selected lines and return the updated cursor position
    pub fn toggle_comment(
        &mut self,
        selection: &Selection,
        comment_token: String,
        lsp_handle: &Option<&mut LSPClientHandle>,
    ) -> Selection {
        self.modified = true;

        if comment_token.is_empty() {
            return *selection;
        }

        let comment_len = comment_token.chars().count();

        let leading_whitespace_len = |row: usize| {
            self.buffer
                .line(row)
                .chars()
                .take_while(|ch| ch.is_whitespace() && *ch != '\n' && *ch != '\r')
                .count()
        };

        let shift_cursor_for_insert = |cursor: &mut Cursor, row: usize, column: usize| {
            if cursor.row == row && cursor.column >= column {
                cursor.column += comment_len;
            }
        };

        let shift_cursor_for_remove = |cursor: &mut Cursor, row: usize, column: usize| {
            if cursor.row == row {
                if cursor.column > column + comment_len {
                    cursor.column -= comment_len;
                } else if cursor.column >= column {
                    cursor.column = column;
                }
            }
        };

        let mut updated_selection = *selection;
        let (start, end) = selection.in_order();
        let indents: Vec<usize> = (start.row..=end.row).map(leading_whitespace_len).collect();
        let uncomment = (start.row..=end.row)
            .zip(indents.iter())
            .all(|(row, indent)| {
                self.buffer
                    .line(row)
                    .chars()
                    .skip(*indent)
                    .take(comment_len)
                    .collect::<String>()
                    == comment_token
            });

        if uncomment {
            for (row, indent) in (start.row..=end.row).zip(indents.iter()) {
                self.remove_text(
                    &Selection {
                        cursor: Cursor {
                            row,
                            column: *indent,
                        },
                        mark: Cursor {
                            row,
                            column: *indent + comment_len,
                        },
                    },
                    lsp_handle,
                    true,
                );
                shift_cursor_for_remove(&mut updated_selection.cursor, row, *indent);
                shift_cursor_for_remove(&mut updated_selection.mark, row, *indent);
            }
        } else {
            for (row, indent) in (start.row..=end.row).zip(indents.iter()) {
                self.insert_text(
                    &comment_token,
                    &Cursor {
                        row,
                        column: *indent,
                    },
                    lsp_handle,
                    true,
                );
                shift_cursor_for_insert(&mut updated_selection.cursor, row, *indent);
                shift_cursor_for_insert(&mut updated_selection.mark, row, *indent);
            }
        }

        updated_selection
    }

    /// Adds line to selection and returns updated selection
    pub fn select_line(&self, selection: &Selection) -> Selection {
        let mut updated_selection = *selection;
        if selection.mark.column == 0
            && selection.cursor.column == self.get_line_length(selection.cursor.row)
            && selection.cursor.row < self.get_num_lines() - 1
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
        let line_chars: Vec<char> = self
            .buffer
            .line(selection.cursor.row)
            .chars()
            .take(self.get_line_length(selection.cursor.row))
            .collect();
        let mut end = selection.cursor.column;
        while end < line_chars.len() && line_chars[end].is_alphanumeric() {
            end += 1;
        }
        updated_selection.cursor.column = end;
        updated_selection
    }

    /// Get word under cursor truncated at the cursor's position
    pub fn get_word_under_cursor(&self, cursor: &Cursor) -> String {
        let line_chars: Vec<char> = self
            .buffer
            .line(cursor.row)
            .chars()
            .take(self.get_line_length(cursor.row))
            .collect();
        let mut start = cursor.column;
        while start > 0 && line_chars[start - 1].is_alphanumeric() {
            start -= 1;
        }
        line_chars[start..cursor.column].iter().collect()
    }

    /// Get range of word under cursor truncated at the cursor's position
    pub fn get_word_range_under_cursor(&self, cursor: &Cursor) -> Selection {
        let line_chars: Vec<char> = self
            .buffer
            .line(cursor.row)
            .chars()
            .take(self.get_line_length(cursor.row))
            .collect();
        let mut start = cursor.column;
        while start > 0 && line_chars[start - 1].is_alphanumeric() {
            start -= 1;
        }
        let start = Cursor {
            row: cursor.row,
            column: start,
        };
        Selection {
            cursor: *cursor,
            mark: start,
        }
    }
}
