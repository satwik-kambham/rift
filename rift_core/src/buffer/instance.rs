use std::collections::HashSet;

use crate::lsp::types;

/// Struct representating a position in the buffer
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub row: usize,
    pub column: usize,
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Self) -> bool {
        self.row == other.row && self.column == other.column
    }
}

impl PartialOrd for Cursor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.row == other.row {
            self.column.partial_cmp(&other.column)
        } else {
            self.row.partial_cmp(&other.row)
        }
    }
}

/// Struct representing a selection where mark is the fixed point / start point
/// of the selection and cursor is the current cursor location which can be
/// moved to update the selection
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Selection {
    pub cursor: Cursor,
    pub mark: Cursor,
}

impl Selection {
    pub fn in_order(&self) -> (&Cursor, &Cursor) {
        if self.cursor >= self.mark {
            return (&self.mark, &self.cursor);
        }
        (&self.cursor, &self.mark)
    }

    pub fn in_order_mut(&mut self) -> (&mut Cursor, &mut Cursor) {
        if self.cursor >= self.mark {
            return (&mut self.mark, &mut self.cursor);
        }
        (&mut self.cursor, &mut self.mark)
    }
}

/// Edit type
#[derive(Debug, Clone)]
pub enum Edit {
    Insert {
        start: Cursor,
        end: Cursor,
        text: String,
    },
    Delete {
        start: Cursor,
        end: Cursor,
        text: String,
    },
}

/// Gutter Information
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct GutterInfo {
    pub start: Cursor,
    pub end: usize,
    pub wrapped: bool,
    pub wrap_end: bool,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Types of highlighted tokens
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum HighlightType {
    None,
    White,
    Red,
    Orange,
    Blue,
    Green,
    Purple,
    Yellow,
    Gray,
    Turquoise,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Attribute {
    None,
    Visible,
    Underline,
    Highlight(HighlightType),
    Select,
    Cursor,
    DiagnosticSeverity(types::DiagnosticSeverity),
}

/// Struct representating a position in the buffer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    pub start: usize,
    pub end: usize,
    pub attributes: HashSet<Attribute>,
}

/// An instance of a buffer (a single buffer can have multiple instances)
/// Contains a cursor for insert mode,
/// a selection for normal / visual mode,
/// scroll position (the line and part of the line at the top of the view)
#[derive(Debug)]
pub struct BufferInstance {
    pub buffer_id: u32,
    pub cursor: Cursor,
    pub selection: Selection,
    pub scroll: Cursor,
    pub column_level: usize,
}

impl BufferInstance {
    pub fn new(buffer_id: u32) -> Self {
        Self {
            buffer_id,
            cursor: Cursor { row: 0, column: 0 },
            selection: Selection {
                cursor: Cursor { row: 0, column: 0 },
                mark: Cursor { row: 0, column: 0 },
            },
            scroll: Cursor { row: 0, column: 0 },
            column_level: 0,
        }
    }

    pub fn set_cursor_position(&mut self, row: usize, column: usize) {
        self.cursor.row = row;
        self.cursor.column = column;
    }

    pub fn set_selection(
        &mut self,
        cursor_row: usize,
        cursor_column: usize,
        mark_row: usize,
        mark_column: usize,
    ) {
        self.selection.cursor.row = cursor_row;
        self.selection.cursor.column = cursor_column;
        self.selection.mark.row = mark_row;
        self.selection.mark.column = mark_column;
    }

    pub fn set_selection_cursor(&mut self, row: usize, column: usize) {
        self.selection.cursor.row = row;
        self.selection.cursor.column = column;
    }

    pub fn set_scroll_position(&mut self, row: usize, column: usize) {
        self.scroll.row = row;
        self.scroll.column = column;
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::instance::{Cursor, Selection};

    #[test]
    fn cursor_eq() {
        let start = Cursor { row: 1, column: 2 };
        let end = Cursor { row: 1, column: 2 };
        assert!(start == end);
    }

    #[test]
    fn cursor_order() {
        let start = Cursor { row: 5, column: 2 };
        let end = Cursor { row: 1, column: 2 };
        assert!(start > end);
    }

    #[test]
    fn selection_order() {
        let start = Cursor { row: 5, column: 2 };
        let end = Cursor { row: 1, column: 2 };
        let selection = Selection {
            mark: end,
            cursor: start,
        };
        let (start_ord, end_ord) = selection.in_order();
        assert_eq!(end, *start_ord);
        assert_eq!(start, *end_ord);
    }
}
