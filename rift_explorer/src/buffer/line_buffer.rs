use std::collections::VecDeque;

use super::highlight;
use super::highlight::LanguageHighlightTypeMapping;

#[derive(serde::Serialize, serde::Deserialize ,Debug, Clone)]
pub struct Cursor {
    pub row: usize,
    pub column: usize,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Language {
    PlainText,
    Python,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Selection {
    pub start: Cursor,
    pub end: Cursor,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Update {
    InsertUpdate {
        start: Cursor,
        end: Cursor,
        text: String,
    },
    RemoveUpdate {
        selection: Selection,
        text: String,
    },
}

/// Basic line based text buffer
pub struct LineTextBuffer {
    pub lines: Vec<String>,
    pub syntax_tree: Option<tree_sitter::Tree>,
    pub highlighted_text: Option<highlight::HighlightedText>,
    pub language: Language,
    pub tokens: Option<Vec<(tree_sitter::Range, String)>>,
    pub updates: VecDeque<Update>,
    pub update_idx: usize,
}

impl LineTextBuffer {
    /// Creates a new line based text buffer from the given initial text
    pub fn new(initial_text: String) -> Self {
        let mut lines: Vec<String> = initial_text.lines().map(String::from).collect();

        if lines.len() == 0 {
            lines.push("".into());
        }

        if initial_text.ends_with("\n") && !initial_text.ends_with("\\n") {
            lines.push("".into());
        }

        Self {
            lines,
            syntax_tree: None,
            highlighted_text: None,
            language: Language::PlainText,
            tokens: None,
            updates: VecDeque::new(),
            update_idx: 0,
        }
    }

    pub fn get_content(&self, eol_sequence: String) -> String {
        let content = self.lines.join(&eol_sequence);
        content
    }

    /// Highlights the entire text in plain text
    pub fn get_highlighted_text(lines: Vec<String>) -> highlight::HighlightedText {
        let mut highlighted_text = highlight::HighlightedText { text: vec![] };

        for line in lines.iter() {
            highlighted_text
                .text
                .push(vec![(highlight::HighlightType::None, line.clone())])
        }

        highlighted_text
    }

    /// Create syntax tree for the current language
    fn create_syntax_tree(&mut self) {
        let mut parser = tree_sitter::Parser::new();
        match self.language {
            Language::Python => {
                parser
                    .set_language(tree_sitter_python::language())
                    .expect("Tree sitter version mismatch");
            }
            _ => {
                return;
            }
        }
        let tree = parser
            .parse_with(
                &mut |_byte: usize, position: tree_sitter::Point| -> &[u8] {
                    let row = position.row as usize;
                    let column = position.column as usize;
                    if row < self.lines.len() {
                        if column < self.lines[row].as_bytes().len() {
                            &self.lines[row].as_bytes()[column..]
                        } else {
                            "\n".as_bytes()
                        }
                    } else {
                        &[]
                    }
                },
                None,
            )
            .unwrap();

        self.syntax_tree = Some(tree);
    }

    /// Gets leaf node information from syntax tree
    fn get_highlighted_tokens(&mut self, cursor: &mut tree_sitter::TreeCursor, parent_kind: &str) {
        let current_kind = parent_kind.to_owned() + "." + cursor.node().kind();
        if cursor.node().child_count() == 0 {
            self.tokens
                .as_mut()
                .unwrap()
                .push((cursor.node().range(), current_kind));
        } else {
            if cursor.goto_first_child() {
                self.get_highlighted_tokens(cursor, &current_kind);
                cursor.goto_parent();
            }
        }

        if cursor.goto_next_sibling() {
            self.get_highlighted_tokens(cursor, parent_kind);
        }
    }

    /// Highlights the entire text for the current language
    pub fn highlight_complete_text(&mut self) -> highlight::HighlightedText {
        self.tokens = Some(vec![]);
        self.create_syntax_tree();
        let syntax_tree = self.syntax_tree.clone().unwrap();
        self.get_highlighted_tokens(&mut syntax_tree.walk(), "root");
        let mut highlighted_text = highlight::HighlightedText { text: vec![] };

        let mapping = highlight::PythonMapping::new();

        let mut tokens_iter = self.tokens.as_ref().unwrap().iter();
        let mut lines_iter = self.lines.iter();

        let mut cursor = Cursor { row: 0, column: 0 };
        let end_cursor = Cursor {
            row: self.get_lines_length() - 1,
            column: self.get_row_length(self.get_lines_length() - 1),
        };
        let mut highlighted_line = vec![];

        let mut current_token = tokens_iter.next();
        let mut current_line = lines_iter.next().unwrap();
        while cursor.row <= end_cursor.row || cursor.column < end_cursor.column {
            match current_token {
                Some((current_range, kind)) => {
                    if cursor.row <= current_range.start_point.row
                        && cursor.column < current_range.start_point.column
                    {
                        // Need to add tokens before as none tokens
                        let mut end_range = self.get_row_length(cursor.row);
                        if cursor.row == current_range.start_point.row {
                            end_range = current_range.start_point.column;
                        }
                        let token_slice = &current_line[cursor.column..end_range];
                        highlighted_line
                            .push((highlight::HighlightType::None, token_slice.to_string()));
                        cursor.column = end_range;
                        if cursor.row < current_range.start_point.row {
                            cursor.row += 1;
                            cursor.column = 0;
                            highlighted_text.text.push(highlighted_line);
                            highlighted_line = vec![];
                            current_line = lines_iter.next().unwrap();
                        }
                    } else {
                        // Add current range
                        let mut end_range = self.get_row_length(cursor.row);
                        if cursor.row == current_range.end_point.row {
                            end_range = current_range.end_point.column;
                        }
                        let token_slice = &current_line[cursor.column..end_range];
                        highlighted_line
                            .push((mapping.get_highlight_type(&kind), token_slice.to_string()));
                        cursor.column = end_range;
                        if cursor.row < current_range.end_point.row && cursor.row < end_cursor.row {
                            cursor.row += 1;
                            cursor.column = 0;
                            highlighted_text.text.push(highlighted_line);
                            highlighted_line = vec![];
                            current_line = lines_iter.next().unwrap();
                        } else if cursor.row == end_cursor.row {
                            highlighted_text.text.push(highlighted_line);
                            break;
                        } else {
                            // Go to next token
                            current_token = tokens_iter.next();
                        }
                    }
                }
                None => {
                    // Add remaining tokens
                    let mut end_range = self.get_row_length(cursor.row);
                    if cursor.row == end_cursor.row {
                        end_range = end_cursor.column;
                    }
                    let token_slice = &current_line[cursor.column..end_range];
                    highlighted_line
                        .push((highlight::HighlightType::None, token_slice.to_string()));
                    cursor.column = end_range;
                    if cursor.row < end_cursor.row {
                        cursor.row += 1;
                        cursor.column = 0;
                        highlighted_text.text.push(highlighted_line);
                        highlighted_line = vec![];
                        current_line = lines_iter.next().unwrap();
                    } else if cursor.row == end_cursor.row {
                        highlighted_text.text.push(highlighted_line);
                        break;
                    }
                }
            }
        }

        highlighted_text
    }

    /// Returns the column length of the given row
    pub fn get_row_length(&self, row: usize) -> usize {
        self.lines[row].len()
    }

    /// Returns the number of lines in the buffer
    pub fn get_lines_length(&self) -> usize {
        self.lines.len()
    }

    pub fn select_token_under_cursor(&self, cursor: Cursor) -> Option<Selection> {
        let tokens = self.tokens.as_ref().unwrap();
        let mut start = None;
        let mut end = None;
        let mut is_identifier = false;
        for (range, kind) in tokens.iter() {
            if cursor.row >= range.start_point.row
                && cursor.row <= range.end_point.row
                && cursor.column >= range.start_point.column
                && cursor.column <= range.end_point.column
            {
                start = Some(Cursor {
                    row: range.start_point.row as usize,
                    column: range.start_point.column as usize,
                });
                end = Some(Cursor {
                    row: range.end_point.row as usize,
                    column: range.end_point.column as usize,
                });
                if kind.ends_with(".identifier") {
                    is_identifier = true;
                }
                break;
            }
        }
        if start.is_some() && end.is_some() {
            if is_identifier {
                return Some(Selection {
                    start: start.unwrap(),
                    end: end.unwrap(),
                });
            }
        }
        None
    }

    /// Insert text at cursor position and returns the updated cursor position
    pub fn insert_text_no_log(&mut self, text: &String, cursor: &Cursor) -> Cursor {
        let mut updated_cursor = cursor.clone();
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

        // Highlight and cache
        self.highlighted_text = Some(self.highlight_complete_text());
        
        updated_cursor
    }

    /// Insert text and log it to updates
    pub fn insert_text(&mut self, text: String, cursor: Cursor) -> Cursor {
        let updated_cursor = self.insert_text_no_log(&text, &cursor);

        self.updates.truncate(self.update_idx);
        self.updates.push_back(Update::InsertUpdate {
            start: cursor,
            end: updated_cursor.clone(),
            text,
        });
        self.update_idx = self.updates.len();

        updated_cursor
    }

    /// Remove the selected text and returns the updated cursor position
    /// and the deleted text
    pub fn remove_text_no_log(&mut self, selection: &Selection) -> (String, Cursor) {
        if selection.start.row == selection.end.row {
            let current_line = self.lines[selection.start.row].clone();
            let (first, second) = current_line.split_at(selection.end.column);
            let (first, middle) = first.split_at(selection.start.column);
            self.lines[selection.start.row] = first.to_owned() + second;

            // Highlight and cache
            self.highlighted_text = Some(self.highlight_complete_text());
        

            (middle.to_owned(), selection.start.clone())
        } else {
            let mut buf = String::new();

            let current_line = self.lines[selection.end.row].clone();
            let (first, second) = current_line.split_at(selection.end.column);
            buf.insert_str(0, first);
            self.lines.remove(selection.end.row);

            for i in (selection.start.row + 1..selection.end.row).rev() {
                let current_line = self.lines.remove(i);
                buf.insert(0, '\n');
                buf.insert_str(0, &current_line);
            }

            let current_line = self.lines[selection.start.row].clone();
            let (first, middle) = current_line.split_at(selection.start.column);
            buf.insert(0, '\n');
            buf.insert_str(0, middle);
            self.lines[selection.start.row] = first.to_owned() + second;

            // Highlight and cache
            self.highlighted_text = Some(self.highlight_complete_text());
        

            (buf, selection.start.clone())
        }
    }

    /// Remove the selected text and log it to updates
    pub fn remove_text(&mut self, selection: Selection) -> (String, Cursor) {
        let (buf, _updated_cursor) = self.remove_text_no_log(&selection);
        self.updates.truncate(self.update_idx);
        self.updates.push_back(Update::RemoveUpdate {
            selection: selection.clone(),
            text: buf.clone(),
        });
        self.update_idx = self.updates.len();

        (buf, selection.start)
    }

    /// Undo last change
    pub fn undo(&mut self) -> Option<Cursor> {
        if self.update_idx > 0 {
            self.update_idx -= 1;
            let update = self.updates.get(self.update_idx).unwrap();
            match update {
                Update::InsertUpdate {
                    start,
                    end,
                    text: _,
                } => {
                    let (_removed_text, updated_cursor) = self.remove_text_no_log(&Selection {
                        start: start.clone(),
                        end: end.clone(),
                    });
                    return Some(updated_cursor);
                }
                Update::RemoveUpdate { selection, text } => {
                    let text = text.clone();
                    let selection = selection.clone();
                    let updated_cursor = self.insert_text_no_log(&text, &selection.start);
                    return Some(updated_cursor);
                }
            }
        }
        None
    }

    /// Redo last change
    pub fn redo(&mut self) -> Option<Cursor> {
        if self.update_idx < self.updates.len() {
            self.update_idx += 1;
            let update = self.updates.get(self.update_idx - 1).unwrap();
            match update {
                Update::InsertUpdate {
                    start,
                    end: _,
                    text,
                } => {
                    let updated_cursor = self.insert_text_no_log(&text.clone(), &start.clone());
                    return Some(updated_cursor);
                }
                Update::RemoveUpdate { selection, text: _ } => {
                    let (_removed_text, updated_cursor) =
                        self.remove_text_no_log(&selection.clone());
                    return Some(updated_cursor);
                }
            }
        }
        None
    }

    /// Add indentation to the selected lines and returns the updated cursor position
    pub fn add_indentation(&mut self, selection: Selection, tab_size: usize) -> Selection {
        let mut updated_selection = selection.clone();
        let tab = " ".repeat(tab_size);
        updated_selection.start.column += tab_size;
        updated_selection.end.column += tab_size;
        for i in selection.start.row..=selection.end.row {
            let current_line = self.lines[i].clone();
            let mut new_line = String::new();
            new_line.push_str(&tab);
            new_line.push_str(&current_line);
            self.lines[i] = new_line;
        }

        // Highlight and cache
        self.highlighted_text = Some(self.highlight_complete_text());
        
        updated_selection
    }

    /// Remove indentation from the selected lines if present and returns the updated cursor position
    pub fn remove_indentation(&mut self, selection: Selection, tab_size: usize) -> Selection {
        let mut updated_selection = selection.clone();
        let tab = " ".repeat(tab_size);
        for i in selection.start.row..=selection.end.row {
            let current_line = self.lines[i].clone();
            if current_line.starts_with(&tab) {
                let (_first, second) = current_line.split_at(tab_size);
                self.lines[i] = second.to_owned();

                if i == selection.start.row {
                    updated_selection.start.column -= tab_size;
                }
                if i == selection.end.row {
                    updated_selection.end.column -= tab_size;
                }
            }
        }

        // Highlight and cache
        self.highlighted_text = Some(self.highlight_complete_text());
        
        updated_selection
    }

    /// Get indent size of the given row
    pub fn get_indent_size(&self, row: usize) -> usize {
        let current_line = self.lines[row].clone();
        let indent_size = current_line.chars().take_while(|c| *c == ' ').count();
        indent_size
    }

    /// Get text at selection
    pub fn get_selected_text(&self, selection: Selection) -> String {
        if selection.start.row == selection.end.row {
            let current_line = self.lines[selection.start.row].clone();
            let (first, _second) = current_line.split_at(selection.end.column);
            let (_first, middle) = first.split_at(selection.start.column);
            middle.to_owned()
        } else {
            let mut buf = String::new();

            let current_line = self.lines[selection.end.row].clone();
            let (first, _second) = current_line.split_at(selection.end.column);
            buf.insert_str(0, first);

            for i in (selection.start.row + 1..selection.end.row).rev() {
                let current_line = self.lines[i].clone();
                buf.insert(0, '\n');
                buf.insert_str(0, &current_line);
            }

            let current_line = self.lines[selection.start.row].clone();
            let (_first, middle) = current_line.split_at(selection.start.column);
            buf.insert(0, '\n');
            buf.insert_str(0, middle);
            buf
        }
    }

    /// Get highlighted line at row
    pub fn get_highlighted_line(&mut self, row: usize) -> highlight::HighlightedLine {
        if self.highlighted_text.is_some() {
            highlight::HighlightedLine {
                text: self.highlighted_text.as_ref().unwrap().text[row].clone()
            }
        } else {
            self.highlighted_text = Some(self.highlight_complete_text());
            highlight::HighlightedLine {
                text: self.highlighted_text.as_ref().unwrap().text[row].clone()
            }
        }
    }
}
