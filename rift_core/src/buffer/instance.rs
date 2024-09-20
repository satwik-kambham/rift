/// Struct representating a position in the buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub row: usize,
    pub column: usize,
}

/// Struct representing a selection where mark is the fixed point / start point
/// of the selection and cursor is the current cursor location which can be
/// moved to update the selection
#[derive(Debug, Clone, Copy)]
pub struct Selection {
    pub cursor: Cursor,
    pub mark: Cursor,
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
