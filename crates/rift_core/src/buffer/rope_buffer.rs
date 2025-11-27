use std::collections::{HashSet, VecDeque};

use crate::lsp::client::LSPClientHandle;

use super::highlight::{TreeSitterParams, build_highlight_params, detect_language};
use super::instance::{Attribute, Cursor, Edit, GutterInfo, Language, Range, Selection};

use ropey::Rope;
use tree_sitter_highlight::Highlighter;

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
        extra_segments: Vec<Range>,
    ) -> (HighlightedText, Cursor, Vec<GutterInfo>) {
        unimplemented!()
    }

    /// Get line length
    pub fn get_line_length(&self, row: usize) -> usize {
        unimplemented!()
    }

    /// Get number of lines
    pub fn get_num_lines(&self) -> usize {
        unimplemented!()
    }

    /// Move cursor right in insert mode
    pub fn move_cursor_right(&self, cursor: &mut Cursor) {
        unimplemented!()
    }

    /// Move cursor left in insert mode
    pub fn move_cursor_left(&self, cursor: &mut Cursor) {
        unimplemented!()
    }

    /// Move cursor up in insert mode
    pub fn move_cursor_up(&self, cursor: &mut Cursor, column_level: usize) -> usize {
        unimplemented!()
    }

    /// Move cursor down in insert mode
    pub fn move_cursor_down(&self, cursor: &mut Cursor, column_level: usize) -> usize {
        unimplemented!()
    }

    /// Move cursor to start of line
    pub fn move_cursor_line_start(&self, cursor: &mut Cursor) {
        unimplemented!()
    }

    /// Move cursor to end of line
    pub fn move_cursor_line_end(&self, cursor: &mut Cursor) {
        unimplemented!()
    }

    /// Move cursor to start of buffer
    pub fn move_cursor_buffer_start(&self, cursor: &mut Cursor) {
        unimplemented!()
    }

    /// Move cursor to end of buffer
    pub fn move_cursor_buffer_end(&self, cursor: &mut Cursor) {
        unimplemented!()
    }

    /// Reset buffer
    pub fn reset(&mut self) {
        self.changes.clear();
        self.change_idx = 0;
    }

    /// Insert text at cursor position and return update cursor position
    pub fn insert_text_no_log(&mut self, text: &str, cursor: &Cursor) -> Cursor {
        unimplemented!()
    }

    pub fn insert_text(
        &mut self,
        text: &str,
        cursor: &Cursor,
        lsp_handle: &Option<&mut LSPClientHandle>,
        log: bool,
    ) -> Cursor {
        unimplemented!()
    }

    /// Removes the selected text and returns the updated cursor position
    /// and the deleted text
    pub fn remove_text_no_log(&mut self, selection: &Selection) -> (String, Cursor) {
        unimplemented!()
    }

    pub fn remove_text(
        &mut self,
        selection: &Selection,
        lsp_handle: &Option<&mut LSPClientHandle>,
        log: bool,
    ) -> (String, Cursor) {
        unimplemented!()
    }

    /// Undo
    pub fn undo(&mut self, lsp_handle: &Option<&mut LSPClientHandle>) -> Option<Cursor> {
        unimplemented!()
    }

    /// Redo
    pub fn redo(&mut self, lsp_handle: &Option<&mut LSPClientHandle>) -> Option<Cursor> {
        unimplemented!()
    }

    /// Get indentation level (number of spaces) of given row
    pub fn get_indentation_level(&self, row: usize) -> usize {
        unimplemented!()
    }

    /// Add indentation to the selected lines and returns the updated cursor position
    pub fn add_indentation(
        &mut self,
        selection: &Selection,
        tab_size: usize,
        lsp_handle: &Option<&mut LSPClientHandle>,
    ) -> Selection {
        unimplemented!()
    }

    /// Remove indentation from the selected lines if present and returns the updated cursor position
    pub fn remove_indentation(
        &mut self,
        selection: &Selection,
        tab_size: usize,
        lsp_handle: &Option<&mut LSPClientHandle>,
    ) -> Selection {
        unimplemented!()
    }

    /// Comment/Uncomment the selected lines and return the updated cursor position
    pub fn toggle_comment(
        &mut self,
        selection: &Selection,
        comment_token: String,
        lsp_handle: &Option<&mut LSPClientHandle>,
    ) -> Selection {
        unimplemented!()
    }

    /// Adds line to selection and returns updated selection
    pub fn select_line(&self, selection: &Selection) -> Selection {
        unimplemented!()
    }

    /// Adds word to selection and returns updated selection
    pub fn select_word(&self, selection: &Selection) -> Selection {
        unimplemented!()
    }

    /// Get word under cursor truncated at the cursor's position
    pub fn get_word_under_cursor(&self, cursor: &Cursor) -> String {
        unimplemented!()
    }

    /// Get range of word under cursor truncated at the cursor's position
    pub fn get_word_range_under_cursor(&self, cursor: &Cursor) -> Selection {
        unimplemented!()
    }
}
