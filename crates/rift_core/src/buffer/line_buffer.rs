use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet, VecDeque},
};

use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use crate::lsp::client::LSPClientHandle;

use super::instance::{
    Attribute, Cursor, Edit, GutterInfo, HighlightType, Language, Range, Selection,
};

/// Tree sitter syntax highlight params
pub struct TreeSitterParams {
    pub language_config: HighlightConfiguration,
    pub highlight_map: HashMap<String, HighlightType>,
    pub highlight_names: Vec<String>,
}

/// Text buffer implementation as a list of lines
pub struct LineBuffer {
    file_path: Option<String>,
    pub display_name: Option<String>,
    pub special: bool,
    pub lines: Vec<String>,
    pub modified: bool,
    pub changes: VecDeque<Edit>,
    pub change_idx: usize,
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

impl LineBuffer {
    fn detect_language(file_path: &Option<String>) -> Language {
        match file_path {
            Some(path) => match std::path::Path::new(&path).extension() {
                Some(extension) => match extension.to_str().unwrap() {
                    "rsl" => Language::RSL,
                    "rs" => Language::Rust,
                    "py" => Language::Python,
                    "md" => Language::Markdown,
                    "toml" => Language::TOML,
                    "nix" => Language::Nix,
                    "dart" => Language::Dart,
                    "html" => Language::HTML,
                    "css" => Language::CSS,
                    "scss" => Language::CSS,
                    "js" => Language::Javascript,
                    "ts" => Language::Typescript,
                    "tsx" => Language::Tsx,
                    "json" => Language::JSON,
                    "c" => Language::C,
                    "h" => Language::C,
                    "cpp" => Language::CPP,
                    "hpp" => Language::CPP,
                    "vue" => Language::Vue,
                    _ => Language::PlainText,
                },
                None => Language::PlainText,
            },
            None => Language::PlainText,
        }
    }

    fn build_highlight_params(language: Language) -> Option<TreeSitterParams> {
        let highlight_map: HashMap<String, HighlightType> = HashMap::from([
            ("attribute".into(), HighlightType::Red),
            ("constant".into(), HighlightType::Red),
            ("constant.builtin".into(), HighlightType::Turquoise),
            ("function.builtin".into(), HighlightType::Purple),
            ("function".into(), HighlightType::Blue),
            ("function.method".into(), HighlightType::Blue),
            ("function.macro".into(), HighlightType::Turquoise),
            ("function.special".into(), HighlightType::Turquoise),
            ("keyword".into(), HighlightType::Purple),
            ("label".into(), HighlightType::Red),
            ("operator".into(), HighlightType::Purple),
            ("property".into(), HighlightType::Yellow),
            ("punctuation".into(), HighlightType::Purple),
            ("punctuation.bracket".into(), HighlightType::Orange),
            ("punctuation.delimiter".into(), HighlightType::Orange),
            ("punctuation.special".into(), HighlightType::Purple),
            ("string".into(), HighlightType::Green),
            ("string.special".into(), HighlightType::Orange),
            ("string.escape".into(), HighlightType::Turquoise),
            ("escape".into(), HighlightType::Turquoise),
            ("comment".into(), HighlightType::Gray),
            ("comment.documentation".into(), HighlightType::Gray),
            ("tag".into(), HighlightType::Blue),
            ("tag.error".into(), HighlightType::Red),
            ("type".into(), HighlightType::Yellow),
            ("type.builtin".into(), HighlightType::Yellow),
            ("variable".into(), HighlightType::Red),
            ("variable.builtin".into(), HighlightType::Orange),
            ("variable.parameter".into(), HighlightType::Red),
            ("text.title".into(), HighlightType::Orange),
            ("text.uri".into(), HighlightType::Blue),
            ("text.reference".into(), HighlightType::Turquoise),
            ("text.literal".into(), HighlightType::Gray),
            ("constructor".into(), HighlightType::Turquoise),
            ("number".into(), HighlightType::Blue),
            ("embedded".into(), HighlightType::Purple),
            ("constructor".into(), HighlightType::Turquoise),
            ("local.definition".into(), HighlightType::Blue),
            ("module".into(), HighlightType::Blue),
        ]);
        let highlight_names: Vec<String> =
            highlight_map.keys().map(|key| key.to_string()).collect();

        let language_config = match language {
            Language::Rust => Some(
                HighlightConfiguration::new(
                    tree_sitter_rust::LANGUAGE.into(),
                    "rust",
                    tree_sitter_rust::HIGHLIGHTS_QUERY,
                    tree_sitter_rust::INJECTIONS_QUERY,
                    "",
                )
                .unwrap(),
            ),
            Language::RSL => Some(
                HighlightConfiguration::new(
                    tree_sitter_rust::LANGUAGE.into(),
                    "rust",
                    tree_sitter_rust::HIGHLIGHTS_QUERY,
                    tree_sitter_rust::INJECTIONS_QUERY,
                    "",
                )
                .unwrap(),
            ),
            Language::Python => Some(
                HighlightConfiguration::new(
                    tree_sitter_python::LANGUAGE.into(),
                    "python",
                    tree_sitter_python::HIGHLIGHTS_QUERY,
                    "",
                    "",
                )
                .unwrap(),
            ),
            Language::Markdown => Some(
                HighlightConfiguration::new(
                    tree_sitter_md::LANGUAGE.into(),
                    "md",
                    tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
                    tree_sitter_md::INJECTION_QUERY_BLOCK,
                    "",
                )
                .unwrap(),
            ),
            Language::Nix => Some(
                HighlightConfiguration::new(
                    tree_sitter_nix::LANGUAGE.into(),
                    "nix",
                    tree_sitter_nix::HIGHLIGHTS_QUERY,
                    "",
                    "",
                )
                .unwrap(),
            ),
            Language::Dart => Some(
                HighlightConfiguration::new(
                    tree_sitter_dart::language(),
                    "dart",
                    tree_sitter_dart::HIGHLIGHTS_QUERY,
                    "",
                    "",
                )
                .unwrap(),
            ),
            Language::HTML => Some(
                HighlightConfiguration::new(
                    tree_sitter_html::LANGUAGE.into(),
                    "html",
                    tree_sitter_html::HIGHLIGHTS_QUERY,
                    tree_sitter_html::INJECTIONS_QUERY,
                    "",
                )
                .unwrap(),
            ),
            Language::CSS => Some(
                HighlightConfiguration::new(
                    tree_sitter_css::LANGUAGE.into(),
                    "css",
                    tree_sitter_css::HIGHLIGHTS_QUERY,
                    "",
                    "",
                )
                .unwrap(),
            ),
            Language::Javascript => Some(
                HighlightConfiguration::new(
                    tree_sitter_javascript::LANGUAGE.into(),
                    "javascript",
                    tree_sitter_javascript::HIGHLIGHT_QUERY,
                    tree_sitter_javascript::INJECTIONS_QUERY,
                    tree_sitter_javascript::LOCALS_QUERY,
                )
                .unwrap(),
            ),
            Language::Typescript => Some(
                HighlightConfiguration::new(
                    tree_sitter_javascript::LANGUAGE.into(),
                    "javascript",
                    tree_sitter_javascript::HIGHLIGHT_QUERY,
                    tree_sitter_javascript::INJECTIONS_QUERY,
                    tree_sitter_javascript::LOCALS_QUERY,
                )
                .unwrap(),
            ),
            Language::Tsx => Some(
                HighlightConfiguration::new(
                    tree_sitter_javascript::LANGUAGE.into(),
                    "javascript",
                    tree_sitter_javascript::HIGHLIGHT_QUERY,
                    tree_sitter_javascript::INJECTIONS_QUERY,
                    tree_sitter_javascript::LOCALS_QUERY,
                )
                .unwrap(),
            ),
            Language::Vue => Some(
                HighlightConfiguration::new(
                    tree_sitter_javascript::LANGUAGE.into(),
                    "javascript",
                    tree_sitter_javascript::HIGHLIGHT_QUERY,
                    tree_sitter_javascript::INJECTIONS_QUERY,
                    tree_sitter_javascript::LOCALS_QUERY,
                )
                .unwrap(),
            ),
            Language::JSON => Some(
                HighlightConfiguration::new(
                    tree_sitter_json::LANGUAGE.into(),
                    "json",
                    tree_sitter_json::HIGHLIGHTS_QUERY,
                    "",
                    "",
                )
                .unwrap(),
            ),
            Language::C => Some(
                HighlightConfiguration::new(
                    tree_sitter_c::LANGUAGE.into(),
                    "c",
                    tree_sitter_c::HIGHLIGHT_QUERY,
                    "",
                    "",
                )
                .unwrap(),
            ),
            Language::CPP => Some(
                HighlightConfiguration::new(
                    tree_sitter_cpp::LANGUAGE.into(),
                    "cpp",
                    &(tree_sitter_c::HIGHLIGHT_QUERY.to_string()
                        + tree_sitter_cpp::HIGHLIGHT_QUERY),
                    "",
                    "",
                )
                .unwrap(),
            ),
            _ => None,
        };

        language_config.map(|mut language_config| {
            language_config.configure(&highlight_names);

            TreeSitterParams {
                language_config,
                highlight_map,
                highlight_names,
            }
        })
    }

    /// Create a line buffer
    pub fn new(
        initial_text: String,
        file_path: Option<String>,
        workspace_folder: &str,
        special: bool,
    ) -> Self {
        let mut lines: Vec<String> = initial_text.lines().map(String::from).collect();

        if let Some(last) = lines.last() {
            if !last.is_empty() {
                lines.push("".into())
            }
        } else {
            lines.push("".into());
        }

        let language = Self::detect_language(&file_path);
        // Syntax highlighter
        let highlighter = Highlighter::new();
        let highlight_params = Self::build_highlight_params(language);

        let mut buffer = Self {
            file_path: None,
            display_name: None,
            special,
            lines,
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

        let language = Self::detect_language(&self.file_path);
        if self.language != language {
            self.language = language;
            self.highlight_params = Self::build_highlight_params(language);
        }
    }

    /// Get text buffer content as a string
    /// with the desired EOL sequence
    pub fn get_content(&self, eol_sequence: String) -> String {
        self.lines.join(&eol_sequence)
    }

    pub fn set_content(&mut self, content: String) {
        let mut lines: Vec<String> = content.lines().map(String::from).collect();

        if let Some(last) = lines.last() {
            if !last.is_empty() {
                lines.push("".into())
            }
        } else {
            lines.push("".into());
        }

        self.lines = lines;
    }

    /// Get a portion text buffer content as a string
    /// with the desired EOL sequence
    pub fn get_content_range(&self, start: usize, end: usize, eol_sequence: String) -> String {
        let range = self.lines.get(start..end + 1).unwrap();
        range.join(&eol_sequence)
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

    pub fn byte_index_from_cursor(&self, cursor: &Cursor, eol_sequence: &str) -> usize {
        let mut byte_index = 0;

        for (idx, line) in self.lines.iter().enumerate() {
            match idx.cmp(&cursor.row) {
                std::cmp::Ordering::Less => {
                    byte_index += line.len() + eol_sequence.len();
                }
                std::cmp::Ordering::Equal => {
                    byte_index += cursor.column;
                }
                std::cmp::Ordering::Greater => {}
            }
        }

        byte_index
    }

    pub fn get_visible_lines(
        &mut self,
        scroll: &mut Cursor,
        cursor: &Cursor,
        selection: &Selection,
        params: &VisibleLineParams,
        mut extra_segments: Vec<Range>,
    ) -> (HighlightedText, Cursor, Vec<GutterInfo>) {
        let max_characters = params.max_characters - 3;
        let mut segments = vec![];
        segments.append(&mut extra_segments);

        // Calculate range of lines which need to be rendered
        // before taking line wrap into account
        let mut range_start = scroll.row;
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

        // Calculate start byte
        let mut start_byte = 0;
        for line in self.lines.get(..range_start).unwrap() {
            start_byte += line.len() + params.eol_sequence.len();
        }

        // Calculate gutter info
        let mut gutter_info = vec![];
        let mut start = 0;
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
                    params.eol_sequence.len()
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
                let end_byte = start_byte + params.eol_sequence.len();
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

        // Add line wrap segments
        for gutter_line in &gutter_info {
            segments.push(Range {
                start: gutter_line.start_byte,
                end: gutter_line.end_byte.saturating_sub(1),
                attributes: HashSet::from([Attribute::Visible]),
            });
        }

        // Calculate relative cursor position
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

            // Update range of lines that need to be rendered
            // taking line wrap into account
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

            scroll.row = gutter_info[range_start].start.row;
            scroll.column = gutter_info[range_start].start.column;

            // Cursor and selection
            let (selection_start, selection_end) = selection.in_order();
            if selection_start != selection_end {
                segments.push(Range {
                    start: self.byte_index_from_cursor(selection_start, &params.eol_sequence),
                    end: self.byte_index_from_cursor(selection_end, &params.eol_sequence),
                    attributes: HashSet::from([Attribute::Select]),
                });
            }

            segments.push(Range {
                start: self.byte_index_from_cursor(cursor, &params.eol_sequence),
                end: self.byte_index_from_cursor(cursor, &params.eol_sequence),
                attributes: HashSet::from([Attribute::Cursor]),
            });
        } else {
            range_start = 0;
            range_end = params.visible_lines;
            range_end = gutter_info.len().min(range_end);
            scroll.row = gutter_info[range_start].start.row;
            scroll.column = gutter_info[range_start].start.column;
        }

        // Highlight
        if let Some(highlight_params) = &self.highlight_params {
            let mut highlight_type = HighlightType::None;
            let content = self.get_content_range(
                gutter_info.first().unwrap().start.row,
                gutter_info.last().unwrap().start.row,
                "\n".into(),
            );
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
                        let start = start + gutter_info.first().unwrap().start_byte;
                        let end = end + gutter_info.first().unwrap().start_byte;
                        if end >= gutter_info.first().unwrap().start_byte
                            && start <= gutter_info.last().unwrap().end_byte
                        {
                            segments.push(Range {
                                start,
                                end: end.saturating_sub(1),
                                attributes: HashSet::from([Attribute::Highlight(highlight_type)]),
                            });
                        }
                    }
                    HighlightEvent::HighlightStart(s) => {
                        highlight_type =
                            highlight_params.highlight_map[&highlight_params.highlight_names[s.0]];
                    }
                    HighlightEvent::HighlightEnd => {
                        highlight_type = HighlightType::None;
                    }
                }
            }
        }

        // Split and render segments
        let mut split_segments = LineBuffer::split_ranges(segments);
        let mut split_segments_iter = split_segments.iter_mut().peekable();
        let mut lines = vec![];
        let mut highlighted_line = vec![];

        while split_segments_iter
            .next_if(|s| s.start < gutter_info.first().unwrap().start_byte)
            .is_some()
        {}

        for line_info in &gutter_info {
            while let Some(segment) = split_segments_iter.next_if(|s| s.end < line_info.end_byte) {
                let line_end = self.lines[line_info.start.row].len();
                let mut buffer_segment = self.lines[line_info.start.row][segment.start
                    - line_info.start_byte
                    + line_info.start.column
                    ..(segment.end - line_info.start_byte + 1 + line_info.start.column)
                        .min(line_end)]
                    .to_string();
                if segment.attributes.contains(&Attribute::Cursor) && buffer_segment.is_empty() {
                    buffer_segment.push(' ');
                }
                let attributes = segment.attributes.clone();
                highlighted_line.push((buffer_segment, attributes));
                if segment.end == line_info.end_byte - 1 {
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

    /// Reset buffer
    pub fn reset(&mut self) {
        self.changes.clear();
        self.change_idx = 0;
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
        let line = &self.lines[row];
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

        let leading_whitespace_len = |line: &str| {
            let mut len = 0;
            for ch in line.chars() {
                if ch.is_whitespace() {
                    len += ch.len_utf8();
                } else {
                    break;
                }
            }
            len
        };

        let shift_cursor_for_insert = |cursor: &mut Cursor, row: usize, column: usize| {
            if cursor.row == row && cursor.column >= column {
                cursor.column += comment_token.len();
            }
        };

        let shift_cursor_for_remove = |cursor: &mut Cursor, row: usize, column: usize| {
            if cursor.row == row {
                if cursor.column > column + comment_token.len() {
                    cursor.column -= comment_token.len();
                } else if cursor.column >= column {
                    cursor.column = column;
                }
            }
        };

        let mut updated_selection = *selection;
        let (start, end) = selection.in_order();
        let indents: Vec<usize> = (start.row..=end.row)
            .map(|row| leading_whitespace_len(&self.lines[row]))
            .collect();
        let uncomment = (start.row..=end.row)
            .zip(indents.iter())
            .all(|(row, indent)| self.lines[row][*indent..].starts_with(&comment_token));

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
                            column: *indent + comment_token.len(),
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

    /// Get word under cursor truncated at the cursor's position
    pub fn get_word_under_cursor(&self, cursor: &Cursor) -> String {
        let line = &self.lines[cursor.row];
        let mut start = cursor.column;
        while start > 0 && line.chars().nth(start - 1).unwrap().is_alphanumeric() {
            start -= 1;
        }
        line[start..cursor.column].to_string()
    }

    /// Get range of word under cursor truncated at the cursor's position
    pub fn get_word_range_under_cursor(&self, cursor: &Cursor) -> Selection {
        let line = &self.lines[cursor.row];
        let mut start = cursor.column;
        while start > 0 && line.chars().nth(start - 1).unwrap().is_alphanumeric() {
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
