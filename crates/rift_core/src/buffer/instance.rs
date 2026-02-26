use crate::lsp::types;
use bitflags::bitflags;

/// Struct representing a position in the buffer
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
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

/// File format / language
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Language {
    PlainText,
    RSL,
    Rust,
    Python,
    Markdown,
    TOML,
    Nix,
    Dart,
    HTML,
    CSS,
    Javascript,
    Typescript,
    Tsx,
    Vue,
    JSON,
    C,
    CPP,
    Zig,
}

/// Types of highlighted tokens
#[derive(
    serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord,
)]
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

bitflags! {
    #[derive(Debug, Clone, Copy, Default, Eq, Hash, PartialEq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
    pub struct TextAttributes: u32 {
        const NONE = 0;
        const VISIBLE = 1 << 0;
        const UNDERLINE = 1 << 1;
        const SELECT = 1 << 2;
        const CURSOR = 1 << 3;

        const HIGHLIGHT_NONE = 1 << 4;
        const HIGHLIGHT_WHITE = 1 << 5;
        const HIGHLIGHT_RED = 1 << 6;
        const HIGHLIGHT_ORANGE = 1 << 7;
        const HIGHLIGHT_BLUE = 1 << 8;
        const HIGHLIGHT_GREEN = 1 << 9;
        const HIGHLIGHT_PURPLE = 1 << 10;
        const HIGHLIGHT_YELLOW = 1 << 11;
        const HIGHLIGHT_GRAY = 1 << 12;
        const HIGHLIGHT_TURQUOISE = 1 << 13;

        const DIAG_HINT = 1 << 14;
        const DIAG_INFORMATION = 1 << 15;
        const DIAG_WARNING = 1 << 16;
        const DIAG_ERROR = 1 << 17;
    }
}

impl TextAttributes {
    pub fn from_highlight(highlight_type: HighlightType) -> Self {
        match highlight_type {
            HighlightType::None => Self::HIGHLIGHT_NONE,
            HighlightType::White => Self::HIGHLIGHT_WHITE,
            HighlightType::Red => Self::HIGHLIGHT_RED,
            HighlightType::Orange => Self::HIGHLIGHT_ORANGE,
            HighlightType::Blue => Self::HIGHLIGHT_BLUE,
            HighlightType::Green => Self::HIGHLIGHT_GREEN,
            HighlightType::Purple => Self::HIGHLIGHT_PURPLE,
            HighlightType::Yellow => Self::HIGHLIGHT_YELLOW,
            HighlightType::Gray => Self::HIGHLIGHT_GRAY,
            HighlightType::Turquoise => Self::HIGHLIGHT_TURQUOISE,
        }
    }

    pub fn from_diagnostic_severity(severity: &types::DiagnosticSeverity) -> Self {
        match severity {
            types::DiagnosticSeverity::Hint => Self::DIAG_HINT,
            types::DiagnosticSeverity::Information => Self::DIAG_INFORMATION,
            types::DiagnosticSeverity::Warning => Self::DIAG_WARNING,
            types::DiagnosticSeverity::Error => Self::DIAG_ERROR,
        }
    }

    pub fn resolve_highlight(self) -> Option<HighlightType> {
        if self.contains(Self::HIGHLIGHT_TURQUOISE) {
            return Some(HighlightType::Turquoise);
        }
        if self.contains(Self::HIGHLIGHT_PURPLE) {
            return Some(HighlightType::Purple);
        }
        if self.contains(Self::HIGHLIGHT_YELLOW) {
            return Some(HighlightType::Yellow);
        }
        if self.contains(Self::HIGHLIGHT_ORANGE) {
            return Some(HighlightType::Orange);
        }
        if self.contains(Self::HIGHLIGHT_BLUE) {
            return Some(HighlightType::Blue);
        }
        if self.contains(Self::HIGHLIGHT_GREEN) {
            return Some(HighlightType::Green);
        }
        if self.contains(Self::HIGHLIGHT_RED) {
            return Some(HighlightType::Red);
        }
        if self.contains(Self::HIGHLIGHT_GRAY) {
            return Some(HighlightType::Gray);
        }
        if self.contains(Self::HIGHLIGHT_WHITE) {
            return Some(HighlightType::White);
        }
        if self.contains(Self::HIGHLIGHT_NONE) {
            return Some(HighlightType::None);
        }
        None
    }

    pub fn has_diagnostic(self) -> bool {
        self.intersects(
            Self::DIAG_HINT | Self::DIAG_INFORMATION | Self::DIAG_WARNING | Self::DIAG_ERROR,
        )
    }

    pub fn resolve_diagnostic_severity(self) -> Option<types::DiagnosticSeverity> {
        if self.contains(Self::DIAG_ERROR) {
            return Some(types::DiagnosticSeverity::Error);
        }
        if self.contains(Self::DIAG_WARNING) {
            return Some(types::DiagnosticSeverity::Warning);
        }
        if self.contains(Self::DIAG_INFORMATION) {
            return Some(types::DiagnosticSeverity::Information);
        }
        if self.contains(Self::DIAG_HINT) {
            return Some(types::DiagnosticSeverity::Hint);
        }
        None
    }
}

/// Struct representing a position in the buffer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    pub start: usize,
    pub end: usize,
    pub attributes: TextAttributes,
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
    use crate::{
        buffer::instance::{Cursor, HighlightType, Selection, TextAttributes},
        lsp::types::DiagnosticSeverity,
    };

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

    #[test]
    fn highlight_resolution_uses_priority() {
        let attrs = TextAttributes::HIGHLIGHT_BLUE | TextAttributes::HIGHLIGHT_TURQUOISE;
        assert_eq!(attrs.resolve_highlight(), Some(HighlightType::Turquoise));
    }

    #[test]
    fn diagnostic_flags_are_detected() {
        let attrs = TextAttributes::from_diagnostic_severity(&DiagnosticSeverity::Warning);
        assert!(attrs.has_diagnostic());
    }

    #[test]
    fn diagnostic_resolution_uses_priority() {
        let attrs = TextAttributes::DIAG_HINT | TextAttributes::DIAG_ERROR;
        assert_eq!(
            attrs.resolve_diagnostic_severity(),
            Some(DiagnosticSeverity::Error)
        );
    }
}
