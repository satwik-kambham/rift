use std::{
    cmp::{max, min},
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::lsp::client::LSPClientHandle;

use super::highlight::{TreeSitterParams, build_highlight_params, detect_language};
use super::instance::{
    Cursor, Edit, GutterInfo, HighlightType, Language, Range, Selection, TextAttributes,
    VirtualSpan,
};

use ropey::Rope;
use tree_sitter::{InputEdit, Point, QueryCursor};

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
    syntax: Option<TreeSitterParams>,
    tree: Option<tree_sitter::Tree>,
    pub input: String,
    pub input_hook: Option<String>,
}

pub type HighlightedText = Vec<Vec<(String, TextAttributes)>>;

struct RopeChunks<'a>(ropey::iter::Chunks<'a>);

impl<'a> Iterator for RopeChunks<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|s| s.as_bytes())
    }
}

/// Parse the rope using the given syntax params, optionally with an old tree for incremental parsing.
fn parse_rope(
    syntax: &mut Option<TreeSitterParams>,
    rope: &Rope,
    old_tree: Option<&tree_sitter::Tree>,
) -> Option<tree_sitter::Tree> {
    let syntax = syntax.as_mut()?;
    let mut callback = |offset: usize, _position: Point| -> &[u8] {
        if offset >= rope.len_bytes() {
            return &[];
        }
        let (chunk, chunk_byte_start, _, _) = rope.chunk_at_byte(offset);
        &chunk.as_bytes()[offset - chunk_byte_start..]
    };
    syntax.parser.parse_with(&mut callback, old_tree)
}

fn cursor_to_point(rope: &Rope, cursor: &Cursor) -> Point {
    let line_byte_start = rope.line_to_byte(cursor.row);
    let char_start = rope.line_to_char(cursor.row);
    let byte_offset = rope.char_to_byte(char_start + cursor.column);
    Point {
        row: cursor.row,
        column: byte_offset - line_byte_start,
    }
}
pub struct VisibleLineParams {
    pub viewport_rows: usize,
    pub viewport_columns: usize,
    pub eol_sequence: String,
}

struct ViewportResult {
    range_start: usize,
    range_end: usize,
    relative_cursor: Cursor,
    segments: Vec<Range>,
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
        let mut syntax = build_highlight_params(language);
        let tree = parse_rope(&mut syntax, &buffer, None);

        let mut buffer = Self {
            file_path: None,
            display_name: None,
            special,
            buffer,
            syntax,
            tree,
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

    pub fn syntax_tree(&self) -> Option<&tree_sitter::Tree> {
        self.tree.as_ref()
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
            self.syntax = build_highlight_params(language);
            self.tree = parse_rope(&mut self.syntax, &self.buffer, None);
        }
    }

    /// Get text buffer content as a string
    /// with the desired EOL sequence
    pub fn get_content(&self, _eol_sequence: String) -> String {
        self.buffer.to_string()
    }

    pub fn set_content(&mut self, content: String) {
        self.buffer = Rope::from_str(&content);
        self.tree = parse_rope(&mut self.syntax, &self.buffer, None);
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

            let mut active_attributes = TextAttributes::empty();
            for range in &ranges {
                if start <= range.end && end >= range.start {
                    active_attributes |= range.attributes;
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

    fn compute_line_range(
        scroll: &Cursor,
        cursor: Option<&Cursor>,
        num_lines: usize,
        viewport_rows: usize,
        special: bool,
    ) -> (usize, usize) {
        let mut range_start = scroll.row.min(num_lines.saturating_sub(1));
        let mut range_end = range_start + viewport_rows + 3;

        if !special && let Some(cursor) = cursor {
            if cursor < scroll {
                range_start = cursor.row;
                range_end = range_start + viewport_rows;
            } else if cursor.row >= scroll.row + viewport_rows {
                range_end = cursor.row + 1;
                range_start = range_end.saturating_sub(viewport_rows);
            }
        }

        (range_start, range_end)
    }

    fn build_gutter_info(
        &self,
        range_start_line: usize,
        range_end_line: usize,
        max_characters: usize,
    ) -> Vec<GutterInfo> {
        let num_lines = self.get_num_lines();
        let mut gutter_info = vec![];
        let end_line = range_end_line.min(num_lines);
        for line_idx in range_start_line..end_line {
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
        gutter_info
    }

    fn build_visibility_segments(gutter_info: &[GutterInfo]) -> Vec<Range> {
        let mut segments = vec![];
        for gutter_line in gutter_info {
            let visible_end = if gutter_line.end_byte > gutter_line.start_byte {
                gutter_line.end_byte - 1
            } else {
                gutter_line.start_byte
            };
            segments.push(Range {
                start: gutter_line.start_byte,
                end: visible_end,
                attributes: TextAttributes::VISIBLE,
            });
        }
        segments
    }

    fn compute_viewport(
        &self,
        gutter_info: &[GutterInfo],
        scroll: &mut Cursor,
        cursor: Option<&Cursor>,
        selection: &Selection,
        viewport_rows: usize,
    ) -> ViewportResult {
        let mut relative_cursor = Cursor { row: 0, column: 0 };
        let mut range_start: usize;
        let mut range_end: usize;
        let mut segments = vec![];

        if !self.special {
            if let Some(cursor) = cursor {
                let mut cursor_idx: usize = 0;
                for line_info in gutter_info {
                    if cursor.row == line_info.start.row
                        && cursor.column >= line_info.start.column
                        && (cursor.column < line_info.end
                            || (cursor.column == line_info.end && line_info.wrap_end))
                    {
                        relative_cursor.column = cursor.column - line_info.start.column;
                        break;
                    }
                    cursor_idx += 1;
                }

                if cursor < scroll {
                    range_start = cursor_idx.saturating_sub(1);
                    range_end = range_start + viewport_rows;
                } else if cursor.row >= scroll.row + viewport_rows {
                    range_end = cursor_idx + 1;
                    range_start = range_end.saturating_sub(viewport_rows);
                } else {
                    range_start = 0;
                    range_end = viewport_rows;
                    if cursor_idx >= viewport_rows {
                        range_end = cursor_idx + 1;
                        range_start = range_end.saturating_sub(viewport_rows);
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
                        attributes: TextAttributes::SELECT,
                    });
                }

                segments.push(Range {
                    start: self.byte_index_from_cursor(cursor),
                    end: self.byte_index_from_cursor(cursor),
                    attributes: TextAttributes::CURSOR,
                });
            } else {
                range_start = 0;
                range_end = viewport_rows;
                range_end = gutter_info.len().min(range_end);

                if !gutter_info.is_empty() {
                    let gutter_len = gutter_info.len();
                    let max_range_start = gutter_len.saturating_sub(1);
                    range_start = range_start.min(max_range_start);
                    if range_start < max_range_start {
                        range_end = (range_start + viewport_rows).min(gutter_len);
                    }

                    scroll.row = gutter_info[range_start].start.row;
                    scroll.column = gutter_info[range_start].start.column;
                }
            }
        } else {
            range_start = 0;
            range_end = viewport_rows;
            range_end = gutter_info.len().min(range_end);
            if !gutter_info.is_empty() {
                scroll.row = gutter_info[range_start].start.row;
                scroll.column = gutter_info[range_start].start.column;
            }
        }

        ViewportResult {
            range_start,
            range_end,
            relative_cursor,
            segments,
        }
    }

    fn compute_highlight_segments(&self, gutter_info: &[GutterInfo]) -> Vec<Range> {
        let mut segments = vec![];
        let (Some(syntax), Some(tree)) = (&self.syntax, &self.tree) else {
            return segments;
        };

        let (Some(first), Some(last)) = (gutter_info.first(), gutter_info.last()) else {
            return segments;
        };

        let start_char = self.buffer.line_to_char(first.start.row);
        let end_line_idx = (last.start.row + 1).min(self.buffer.len_lines());
        let end_char = self.buffer.line_to_char(end_line_idx);

        let start_byte = self.buffer.char_to_byte(start_char);
        let end_byte = self.buffer.char_to_byte(end_char);

        let mut cursor = QueryCursor::new();
        cursor.set_byte_range(start_byte..end_byte);

        let root = tree.root_node();
        let text_callback = |node: tree_sitter::Node| {
            let start = node.start_byte();
            let end = node.end_byte();
            let start_char = self.buffer.byte_to_char(start);
            let end_char = self.buffer.byte_to_char(end);
            RopeChunks(self.buffer.slice(start_char..end_char).chunks())
        };

        for (m, capture_idx) in cursor.captures(&syntax.highlight_query, root, text_callback) {
            let capture = m.captures[capture_idx];
            let highlight_type = syntax.capture_map[capture.index as usize];
            if matches!(highlight_type, HighlightType::None) {
                continue;
            }

            let node = capture.node;
            let node_start = self.buffer.byte_to_char(node.start_byte());
            let node_end = self.buffer.byte_to_char(node.end_byte());

            if node_end >= first.start_byte && node_start <= last.end_byte {
                segments.push(Range {
                    start: node_start,
                    end: node_end.saturating_sub(1),
                    attributes: TextAttributes::from_highlight(highlight_type),
                });
            }
        }

        segments
    }

    fn merge_and_extract_text(
        &self,
        gutter_info: &[GutterInfo],
        segments: Vec<Range>,
    ) -> HighlightedText {
        let mut split_segments = RopeBuffer::split_ranges(segments);
        let mut split_segments_iter = split_segments.iter_mut().peekable();
        let mut lines = vec![];
        let mut highlighted_line = vec![];

        if let Some(first) = gutter_info.first() {
            while split_segments_iter
                .next_if(|s| s.start < first.start_byte)
                .is_some()
            {}
        }

        for line_info in gutter_info {
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
                if segment.attributes.contains(TextAttributes::CURSOR) && buffer_segment.is_empty()
                {
                    buffer_segment.push(' ');
                }
                let attributes = segment.attributes;
                highlighted_line.push((buffer_segment, attributes));
                if segment.end == line_info.end_byte.saturating_sub(1) {
                    lines.push(highlighted_line);
                    highlighted_line = vec![];
                }
            }
        }

        lines
    }

    /// Inject virtual spans into the already-extracted HighlightedText.
    /// Returns (extra_rows_before_cursor, extra_columns_on_cursor_line).
    fn inject_virtual_spans(
        lines: &mut HighlightedText,
        gutter_info: &mut Vec<GutterInfo>,
        virtual_spans: &[VirtualSpan],
        cursor_row: usize,
        cursor_col: usize,
    ) -> (usize, usize) {
        if virtual_spans.is_empty() {
            return (0, 0);
        }

        // Sort by position descending so later insertions don't shift earlier indices
        let mut sorted: Vec<&VirtualSpan> = virtual_spans.iter().collect();
        sorted.sort_by(|a, b| {
            b.position
                .row
                .cmp(&a.position.row)
                .then(b.position.column.cmp(&a.position.column))
        });

        let mut extra_lines_before_cursor = 0;
        let mut extra_cols_on_cursor_line = 0;

        for span in &sorted {
            if span.text.is_empty() {
                continue;
            }

            // Find the gutter_info entry matching this span's buffer row
            let gutter_idx = gutter_info.iter().position(|g| {
                !g.wrapped
                    && g.start.row == span.position.row
                    && span.position.column >= g.start.column
            });

            let Some(gutter_idx) = gutter_idx else {
                continue;
            };

            // For wrapped lines, find the correct segment
            let mut target_idx = gutter_idx;
            for (i, g) in gutter_info.iter().enumerate().skip(gutter_idx) {
                if g.start.row != span.position.row {
                    break;
                }
                if span.position.column >= g.start.column
                    && span.position.column < g.start.column + (g.end - g.start.column)
                {
                    target_idx = i;
                    break;
                }
                target_idx = i;
            }

            if target_idx >= lines.len() {
                continue;
            }

            let line = &lines[target_idx];
            let relative_col = span
                .position
                .column
                .saturating_sub(gutter_info[target_idx].start.column);

            // Walk tokens to find the split point
            let mut char_count = 0;
            let mut split_token_idx = line.len();
            let mut split_char_offset = 0;

            for (idx, (text, attrs)) in line.iter().enumerate() {
                // Skip previously-injected virtual tokens when counting
                if attrs.contains(TextAttributes::VIRTUAL) {
                    continue;
                }
                let token_chars = text.chars().count();
                if char_count + token_chars > relative_col {
                    split_token_idx = idx;
                    split_char_offset = relative_col - char_count;
                    break;
                }
                char_count += token_chars;
                if char_count >= relative_col {
                    split_token_idx = idx + 1;
                    split_char_offset = 0;
                    break;
                }
            }

            // Advance past any consecutive VIRTUAL tokens at the split point
            // so new virtual text groups after previously-injected virtual text
            while split_token_idx < line.len()
                && line[split_token_idx].1.contains(TextAttributes::VIRTUAL)
            {
                split_token_idx += 1;
                split_char_offset = 0;
            }

            let virtual_fragments: Vec<&str> = span.text.split('\n').collect();
            let virtual_attrs = span.attributes | TextAttributes::VIRTUAL;

            if virtual_fragments.len() == 1 {
                // Single-line: splice virtual text inline
                let mut new_line = Vec::new();
                for (idx, (text, attrs)) in lines[target_idx].iter().enumerate() {
                    if idx == split_token_idx && split_char_offset > 0 {
                        let (before, after) = split_string_at_char(text, split_char_offset);
                        if !before.is_empty() {
                            new_line.push((before, *attrs));
                        }
                        new_line.push((virtual_fragments[0].to_string(), virtual_attrs));
                        if !after.is_empty() {
                            new_line.push((after, *attrs));
                        }
                    } else if idx == split_token_idx && split_char_offset == 0 {
                        // The EOL cursor placeholder is a synthetic " " with CURSOR
                        // at the end of the line — virtual text goes after it so the
                        // cursor stays at its buffer position.
                        let is_eol_cursor = text == " "
                            && attrs.contains(TextAttributes::CURSOR)
                            && idx == lines[target_idx].len() - 1;
                        if is_eol_cursor {
                            // Drop the synthetic space and instead apply CURSOR
                            // to the first char of the virtual text so the cursor
                            // renders on top of the virtual span without shifting it.
                            let vtext = virtual_fragments[0].to_string();
                            if let Some(first_ch) = vtext.chars().next() {
                                new_line.push((
                                    first_ch.to_string(),
                                    virtual_attrs | TextAttributes::CURSOR,
                                ));
                                let rest: String = vtext.chars().skip(1).collect();
                                if !rest.is_empty() {
                                    new_line.push((rest, virtual_attrs));
                                }
                            }
                        } else {
                            new_line.push((virtual_fragments[0].to_string(), virtual_attrs));
                            new_line.push((text.clone(), *attrs));
                        }
                    } else {
                        new_line.push((text.clone(), *attrs));
                    }
                }
                // If split point is at/past the end of the line, append
                if split_token_idx >= lines[target_idx].len() {
                    new_line.push((virtual_fragments[0].to_string(), virtual_attrs));
                }
                // Track extra columns if this span is on the cursor line and
                // before the cursor column (strictly less than — a span at the
                // cursor column itself does not push the cursor right)
                if target_idx == cursor_row && span.position.column < cursor_col {
                    extra_cols_on_cursor_line += virtual_fragments[0].chars().count();
                }
                lines[target_idx] = new_line;
            } else {
                // Multi-line: first fragment goes inline, rest become new lines
                let original_line = lines[target_idx].clone();
                let (before_tokens, after_tokens) =
                    split_tokens_at(&original_line, split_token_idx, split_char_offset);

                // First line: before + first fragment
                let mut first_line = before_tokens;
                first_line.push((virtual_fragments[0].to_string(), virtual_attrs));
                lines[target_idx] = first_line;

                // Middle virtual lines
                let gutter_template = GutterInfo {
                    start: span.position,
                    end: 0,
                    wrapped: true,
                    wrap_end: true,
                    start_byte: 0,
                    end_byte: 0,
                };

                let mut inserted = 0;
                for frag in &virtual_fragments[1..virtual_fragments.len() - 1] {
                    inserted += 1;
                    let insert_at = target_idx + inserted;
                    lines.insert(insert_at, vec![(frag.to_string(), virtual_attrs)]);
                    gutter_info.insert(insert_at, gutter_template);
                }

                // Last fragment + remaining real tokens
                inserted += 1;
                let insert_at = target_idx + inserted;
                let mut last_line =
                    vec![(virtual_fragments.last().unwrap().to_string(), virtual_attrs)];
                last_line.extend(after_tokens);
                lines.insert(insert_at, last_line);
                gutter_info.insert(insert_at, gutter_template);

                // Count extra lines inserted before cursor
                if target_idx < cursor_row {
                    extra_lines_before_cursor += inserted;
                } else if target_idx == cursor_row {
                    // Virtual span is on the cursor line — extra lines push cursor down
                    extra_lines_before_cursor += inserted;
                }
            }
        }

        (extra_lines_before_cursor, extra_cols_on_cursor_line)
    }

    /// Wrap any lines exceeding max_characters into continuation lines.
    /// Returns (total_extra_lines, extra_rows_before_cursor, Option<new_cursor_col>).
    fn wrap_lines_at_boundary(
        lines: &mut HighlightedText,
        gutter_info: &mut Vec<GutterInfo>,
        max_characters: usize,
        mut cursor_row: usize,
        cursor_col: usize,
    ) -> (usize, usize, Option<usize>) {
        let mut total_extra = 0usize;
        let mut extra_before_cursor = 0usize;
        let mut new_cursor_col: Option<usize> = None;
        let mut i = 0;

        while i < lines.len() {
            let width: usize = lines[i].iter().map(|(t, _)| t.chars().count()).sum();
            if width <= max_characters {
                i += 1;
                continue;
            }

            // Split tokens at the max_characters boundary
            let mut char_count = 0;
            let mut split_token = lines[i].len();
            let mut split_offset = 0;

            for (idx, (text, _)) in lines[i].iter().enumerate() {
                let token_chars = text.chars().count();
                if char_count + token_chars > max_characters {
                    split_token = idx;
                    split_offset = max_characters - char_count;
                    break;
                }
                char_count += token_chars;
                if char_count == max_characters {
                    split_token = idx + 1;
                    split_offset = 0;
                    break;
                }
            }

            // Build the two halves
            let mut keep = Vec::new();
            let mut overflow = Vec::new();

            for (idx, (text, attrs)) in lines[i].iter().enumerate() {
                if idx < split_token {
                    keep.push((text.clone(), *attrs));
                } else if idx == split_token && split_offset > 0 {
                    let (before, after) = split_string_at_char(text, split_offset);
                    if !before.is_empty() {
                        keep.push((before, *attrs));
                    }
                    if !after.is_empty() {
                        overflow.push((after, *attrs));
                    }
                } else {
                    overflow.push((text.clone(), *attrs));
                }
            }

            // Update gutter_info
            let original_wrap_end = gutter_info[i].wrap_end;
            gutter_info[i].wrap_end = false;

            let continuation_gutter = GutterInfo {
                start: gutter_info[i].start,
                end: gutter_info[i].end,
                wrapped: true,
                wrap_end: original_wrap_end,
                start_byte: 0,
                end_byte: 0,
            };

            lines[i] = keep;
            lines.insert(i + 1, overflow);
            gutter_info.insert(i + 1, continuation_gutter);
            total_extra += 1;

            // Track cursor adjustments
            if i < cursor_row {
                extra_before_cursor += 1;
                cursor_row += 1;
            } else if i == cursor_row {
                // Check if the cursor falls into the overflow portion
                if cursor_col >= max_characters {
                    extra_before_cursor += 1;
                    cursor_row += 1;
                    // The cursor is now on the next line with adjusted column
                    let adjusted_col = new_cursor_col.unwrap_or(cursor_col) - max_characters;
                    new_cursor_col = Some(adjusted_col);
                }
            }

            // Do NOT advance i — the overflow line itself may need further splitting
            // But if we didn't split (shouldn't happen), advance to avoid infinite loop
            if lines[i].is_empty() {
                i += 1;
            }
        }

        (total_extra, extra_before_cursor, new_cursor_col)
    }

    fn slice_viewport(
        lines: HighlightedText,
        gutter_info: Vec<GutterInfo>,
        range_start: usize,
        range_end: usize,
    ) -> (HighlightedText, Vec<GutterInfo>) {
        (
            lines
                .get(range_start..range_end)
                .unwrap_or(&lines[range_start..])
                .to_vec(),
            gutter_info
                .get(range_start..range_end)
                .unwrap_or(&gutter_info[range_start..])
                .to_vec(),
        )
    }

    pub fn get_visible_lines(
        &mut self,
        scroll: &mut Cursor,
        cursor: Option<&Cursor>,
        selection: &Selection,
        params: &VisibleLineParams,
        mut extra_segments: Vec<Range>,
        virtual_spans: &[VirtualSpan],
    ) -> (HighlightedText, Cursor, Vec<GutterInfo>) {
        let max_characters = params.viewport_columns.saturating_sub(3).max(1);
        let num_lines = self.get_num_lines();

        let (start_line, end_line) = Self::compute_line_range(
            scroll,
            cursor,
            num_lines,
            params.viewport_rows,
            self.special,
        );
        let mut gutter_info = self.build_gutter_info(start_line, end_line, max_characters);

        let viewport = self.compute_viewport(
            &gutter_info,
            scroll,
            cursor,
            selection,
            params.viewport_rows,
        );
        extra_segments.extend(viewport.segments);
        extra_segments.extend(Self::build_visibility_segments(&gutter_info));

        let highlight_segments = self.compute_highlight_segments(&gutter_info);
        extra_segments.extend(highlight_segments);

        let mut lines = self.merge_and_extract_text(&gutter_info, extra_segments);

        let (extra_rows, extra_cols) = Self::inject_virtual_spans(
            &mut lines,
            &mut gutter_info,
            virtual_spans,
            viewport.relative_cursor.row + viewport.range_start,
            viewport.relative_cursor.column,
        );

        // Wrap any lines that now exceed max_characters due to virtual text
        let adjusted_cursor_row = viewport.relative_cursor.row + viewport.range_start + extra_rows;
        let adjusted_cursor_col = viewport.relative_cursor.column + extra_cols;
        let (wrap_total, wrap_before_cursor, wrap_new_col) = Self::wrap_lines_at_boundary(
            &mut lines,
            &mut gutter_info,
            max_characters,
            adjusted_cursor_row,
            adjusted_cursor_col,
        );

        let (lines, gutter_info) = Self::slice_viewport(
            lines,
            gutter_info,
            viewport.range_start,
            viewport.range_end + extra_rows + wrap_total,
        );

        let mut relative_cursor = viewport.relative_cursor;
        relative_cursor.row += extra_rows + wrap_before_cursor;
        relative_cursor.column = match wrap_new_col {
            Some(col) => col,
            None => viewport.relative_cursor.column + extra_cols,
        };

        (lines, relative_cursor, gutter_info)
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
        updated_cursor.column += current_line_part.chars().count();
        for segment in text_iter {
            updated_cursor.row += 1;
            updated_cursor.column = segment.chars().count();
        }

        let char_idx = self.buffer.line_to_char(cursor.row) + cursor.column;
        let start_byte = self.buffer.char_to_byte(char_idx);
        let start_position = cursor_to_point(&self.buffer, cursor);

        self.buffer.insert(char_idx, text);

        if let Some(tree) = &mut self.tree {
            let new_end_byte = start_byte + text.len();
            let new_end_position = cursor_to_point(&self.buffer, &updated_cursor);
            tree.edit(&InputEdit {
                start_byte,
                old_end_byte: start_byte,
                new_end_byte,
                start_position,
                old_end_position: start_position,
                new_end_position,
            });
            self.tree = parse_rope(&mut self.syntax, &self.buffer, self.tree.as_ref());
        }

        updated_cursor
    }

    pub fn insert_text(
        &mut self,
        text: &str,
        cursor: &Cursor,
        lsp_handle: &Option<Arc<Mutex<LSPClientHandle>>>,
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
            let sync_kind =
                if lsp_handle.lock().unwrap().initialize_capabilities["textDocumentSync"].is_u64() {
                    lsp_handle.lock().unwrap().initialize_capabilities["textDocumentSync"]
                        .as_u64()
                        .unwrap()
                } else if lsp_handle.lock().unwrap().initialize_capabilities["textDocumentSync"]["change"].is_u64()
                {
                    lsp_handle.lock().unwrap().initialize_capabilities["textDocumentSync"]["change"]
                        .as_u64()
                        .unwrap()
                } else {
                    0
                };

            if sync_kind != 0 {
                if sync_kind == 1 {
                    lsp_handle
                        .lock()
                        .unwrap()
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
                        .lock()
                        .unwrap()
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

        let start_byte = self.buffer.char_to_byte(start_idx);
        let old_end_byte = self.buffer.char_to_byte(end_idx);
        let start_position = cursor_to_point(&self.buffer, start);
        let old_end_position = cursor_to_point(&self.buffer, end);

        let deleted_text = self.buffer.slice(start_idx..end_idx).to_string();
        self.buffer.remove(start_idx..end_idx);

        if let Some(tree) = &mut self.tree {
            tree.edit(&InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte: start_byte,
                start_position,
                old_end_position,
                new_end_position: start_position,
            });
            self.tree = parse_rope(&mut self.syntax, &self.buffer, self.tree.as_ref());
        }

        (deleted_text, *start)
    }

    pub fn remove_text(
        &mut self,
        selection: &Selection,
        lsp_handle: &Option<Arc<Mutex<LSPClientHandle>>>,
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
            let sync_kind = if lsp_handle.lock().unwrap().initialize_capabilities["textDocumentSync"].is_u64() {
                lsp_handle.lock().unwrap().initialize_capabilities["textDocumentSync"]
                    .as_u64()
                    .unwrap()
            } else if lsp_handle.lock().unwrap().initialize_capabilities["textDocumentSync"]["change"].is_u64() {
                lsp_handle.lock().unwrap().initialize_capabilities["textDocumentSync"]["change"]
                    .as_u64()
                    .unwrap()
            } else {
                0
            };

            if sync_kind != 0 {
                if sync_kind == 1 {
                    lsp_handle
                        .lock()
                        .unwrap()
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
                        .lock()
                        .unwrap()
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
    pub fn undo(&mut self, lsp_handle: &Option<Arc<Mutex<LSPClientHandle>>>) -> Option<Cursor> {
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
    pub fn redo(&mut self, lsp_handle: &Option<Arc<Mutex<LSPClientHandle>>>) -> Option<Cursor> {
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
        lsp_handle: &Option<Arc<Mutex<LSPClientHandle>>>,
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
        lsp_handle: &Option<Arc<Mutex<LSPClientHandle>>>,
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
        lsp_handle: &Option<Arc<Mutex<LSPClientHandle>>>,
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
        updated_selection.mark.column = 0;
        if updated_selection.cursor.row < self.get_num_lines() - 1 {
            updated_selection.cursor.row += 1;
            updated_selection.cursor.column = 0;
        } else {
            updated_selection.cursor.column = self.get_line_length(updated_selection.cursor.row);
        }
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

    /// Find the next occurrence of the given query starting at the provided cursor.
    /// Returns a selection covering the match if found.
    pub fn find_next(&self, cursor: &Cursor, query: &str) -> Option<Selection> {
        if query.is_empty() {
            return None;
        }

        let total_chars = self.buffer.len_chars();
        let start_idx = self.byte_index_from_cursor(cursor).min(total_chars);
        if start_idx == total_chars {
            return None;
        }

        let search_slice = self.buffer.slice(start_idx..total_chars).to_string();
        let match_offset_bytes = search_slice.find(query)?;
        let leading_chars = search_slice[..match_offset_bytes].chars().count();
        let match_start = start_idx + leading_chars;
        let match_end = match_start + query.chars().count();

        let start_row = self.buffer.char_to_line(match_start);
        let start_col = match_start - self.buffer.line_to_char(start_row);
        let end_row = self.buffer.char_to_line(match_end);
        let end_col = match_end - self.buffer.line_to_char(end_row);

        Some(Selection {
            mark: Cursor {
                row: start_row,
                column: start_col,
            },
            cursor: Cursor {
                row: end_row,
                column: end_col,
            },
        })
    }
}

/// Split a string at a character offset, returning (before, after).
fn split_string_at_char(s: &str, char_offset: usize) -> (String, String) {
    let byte_offset = s
        .char_indices()
        .nth(char_offset)
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    (s[..byte_offset].to_string(), s[byte_offset..].to_string())
}

type TokenLine = Vec<(String, TextAttributes)>;

/// Split a token list at a specific token index and character offset within that token.
fn split_tokens_at(
    tokens: &[(String, TextAttributes)],
    token_idx: usize,
    char_offset: usize,
) -> (TokenLine, TokenLine) {
    let mut before = Vec::new();
    let mut after = Vec::new();

    for (idx, (text, attrs)) in tokens.iter().enumerate() {
        if idx < token_idx {
            before.push((text.clone(), *attrs));
        } else if idx == token_idx && char_offset > 0 {
            let (b, a) = split_string_at_char(text, char_offset);
            if !b.is_empty() {
                before.push((b, *attrs));
            }
            if !a.is_empty() {
                after.push((a, *attrs));
            }
        } else {
            after.push((text.clone(), *attrs));
        }
    }

    (before, after)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_ranges_merges_attributes_with_bitwise_or() {
        let ranges = vec![
            Range {
                start: 0,
                end: 2,
                attributes: TextAttributes::SELECT,
            },
            Range {
                start: 1,
                end: 3,
                attributes: TextAttributes::CURSOR,
            },
        ];

        let split = RopeBuffer::split_ranges(ranges);
        assert_eq!(split.len(), 3);
        assert_eq!(split[0].start, 0);
        assert_eq!(split[0].end, 0);
        assert!(split[0].attributes.contains(TextAttributes::SELECT));
        assert!(!split[0].attributes.contains(TextAttributes::CURSOR));

        assert_eq!(split[1].start, 1);
        assert_eq!(split[1].end, 2);
        assert!(split[1].attributes.contains(TextAttributes::SELECT));
        assert!(split[1].attributes.contains(TextAttributes::CURSOR));

        assert_eq!(split[2].start, 3);
        assert_eq!(split[2].end, 3);
        assert!(!split[2].attributes.contains(TextAttributes::SELECT));
        assert!(split[2].attributes.contains(TextAttributes::CURSOR));
    }

    #[test]
    fn find_same_line_from_start() {
        let buffer = RopeBuffer::new("hello world\nbye".to_string(), None, ".", false);
        let result = buffer.find_next(&Cursor { row: 0, column: 0 }, "hello");
        let selection = result.expect("should find match at start");
        assert_eq!(selection.mark, Cursor { row: 0, column: 0 });
        assert_eq!(selection.cursor, Cursor { row: 0, column: 5 });
    }

    #[test]
    fn find_after_cursor_multi_line() {
        let buffer = RopeBuffer::new("abc\ndef abc".to_string(), None, ".", false);
        let result = buffer.find_next(&Cursor { row: 0, column: 1 }, "abc");
        let selection = result.expect("should find second occurrence");
        assert_eq!(selection.mark, Cursor { row: 1, column: 4 });
        assert_eq!(selection.cursor, Cursor { row: 1, column: 7 });
    }

    #[test]
    fn find_returns_none_when_missing() {
        let buffer = RopeBuffer::new("foo bar".to_string(), None, ".", false);
        let result = buffer.find_next(&Cursor { row: 0, column: 0 }, "baz");
        assert!(result.is_none());
    }

    #[test]
    fn find_does_not_wrap() {
        let buffer = RopeBuffer::new("match here\nstart".to_string(), None, ".", false);
        let result = buffer.find_next(&Cursor { row: 1, column: 0 }, "match");
        assert!(result.is_none(), "should not wrap to earlier lines");
    }
}
