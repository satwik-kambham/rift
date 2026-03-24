use super::instance::{Cursor, Range, Selection, TextAttributes};
use super::rope_buffer::{HighlightedText, RopeBuffer, VisibleLineParams};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn buf(text: &str) -> RopeBuffer {
    RopeBuffer::new(text.to_string(), None, ".", false)
}

fn cur(row: usize, column: usize) -> Cursor {
    Cursor { row, column }
}

fn sel(mr: usize, mc: usize, cr: usize, cc: usize) -> Selection {
    Selection {
        mark: cur(mr, mc),
        cursor: cur(cr, cc),
    }
}

fn content(b: &RopeBuffer) -> String {
    b.get_content("\n".to_string())
}

// ── Group 1: Text Mutation Primitives ────────────────────────────────────────

#[test]
fn insert_at_start() {
    let mut b = buf("hello");
    let c = b.insert_text_no_log("XY", &cur(0, 0));
    assert_eq!(c, cur(0, 2));
    assert_eq!(content(&b), "XYhello");
}

#[test]
fn insert_mid_line() {
    let mut b = buf("hello world");
    let c = b.insert_text_no_log("XY", &cur(0, 5));
    assert_eq!(c, cur(0, 7));
    assert_eq!(content(&b), "helloXY world");
}

#[test]
fn insert_at_end_of_line() {
    let mut b = buf("abc\ndef");
    let c = b.insert_text_no_log("Z", &cur(0, 3));
    assert_eq!(c, cur(0, 4));
    assert_eq!(content(&b), "abcZ\ndef");
}

#[test]
fn insert_newline() {
    let mut b = buf("abcdef");
    let c = b.insert_text_no_log("\n", &cur(0, 3));
    assert_eq!(c, cur(1, 0));
    assert_eq!(content(&b), "abc\ndef");
}

#[test]
fn insert_multiline_text() {
    let mut b = buf("start end");
    let c = b.insert_text_no_log("A\nBB\nCCC", &cur(0, 6));
    assert_eq!(c, cur(2, 3));
    assert_eq!(content(&b), "start A\nBB\nCCCend");
}

#[test]
fn insert_into_empty_buffer() {
    let mut b = buf("");
    let c = b.insert_text_no_log("hi", &cur(0, 0));
    assert_eq!(c, cur(0, 2));
    assert_eq!(content(&b), "hi");
}

#[test]
fn remove_single_char() {
    let mut b = buf("abcd");
    let (deleted, c) = b.remove_text_no_log(&sel(0, 1, 0, 2));
    assert_eq!(deleted, "b");
    assert_eq!(c, cur(0, 1));
    assert_eq!(content(&b), "acd");
}

#[test]
fn remove_across_lines() {
    let mut b = buf("abc\ndef");
    let (deleted, c) = b.remove_text_no_log(&sel(0, 2, 1, 1));
    assert_eq!(deleted, "c\nd");
    assert_eq!(c, cur(0, 2));
    assert_eq!(content(&b), "abef");
}

#[test]
fn remove_entire_content() {
    let mut b = buf("abc\ndef");
    let (deleted, c) = b.remove_text_no_log(&sel(0, 0, 1, 3));
    assert_eq!(deleted, "abc\ndef");
    assert_eq!(c, cur(0, 0));
    assert_eq!(content(&b), "");
}

#[test]
fn remove_backward_selection() {
    let mut b = buf("abcd");
    // mark after cursor — same result as forward
    let (deleted, c) = b.remove_text_no_log(&sel(0, 3, 0, 1));
    assert_eq!(deleted, "bc");
    assert_eq!(c, cur(0, 1));
    assert_eq!(content(&b), "ad");
}

#[test]
fn insert_sets_modified_flag() {
    let mut b = buf("x");
    assert!(!b.modified);
    b.insert_text_no_log("y", &cur(0, 0));
    assert!(b.modified);
}

#[test]
fn remove_sets_modified_flag() {
    let mut b = buf("xy");
    assert!(!b.modified);
    b.remove_text_no_log(&sel(0, 0, 0, 1));
    assert!(b.modified);
}

// ── Group 2: Cursor Movement ─────────────────────────────────────────────────

#[test]
fn move_right_mid_line() {
    let b = buf("hello");
    let mut c = cur(0, 2);
    b.move_cursor_right(&mut c);
    assert_eq!(c, cur(0, 3));
}

#[test]
fn move_right_end_of_line_wraps() {
    let b = buf("ab\ncd");
    let mut c = cur(0, 2);
    b.move_cursor_right(&mut c);
    assert_eq!(c, cur(1, 0));
}

#[test]
fn move_right_end_of_buffer_noop() {
    let b = buf("ab");
    let mut c = cur(0, 2);
    b.move_cursor_right(&mut c);
    assert_eq!(c, cur(0, 2));
}

#[test]
fn move_left_mid_line() {
    let b = buf("hello");
    let mut c = cur(0, 3);
    b.move_cursor_left(&mut c);
    assert_eq!(c, cur(0, 2));
}

#[test]
fn move_left_start_of_line_wraps() {
    let b = buf("ab\ncd");
    let mut c = cur(1, 0);
    b.move_cursor_left(&mut c);
    assert_eq!(c, cur(0, 2));
}

#[test]
fn move_left_start_of_buffer_noop() {
    let b = buf("ab");
    let mut c = cur(0, 0);
    b.move_cursor_left(&mut c);
    assert_eq!(c, cur(0, 0));
}

#[test]
fn move_up_normal() {
    let b = buf("abcde\nfghij");
    let mut c = cur(1, 3);
    b.move_cursor_up(&mut c, 3);
    assert_eq!(c, cur(0, 3));
}

#[test]
fn move_up_clamps_to_shorter_line() {
    let b = buf("ab\nfghij");
    let mut c = cur(1, 4);
    b.move_cursor_up(&mut c, 4);
    assert_eq!(c, cur(0, 2));
}

#[test]
fn move_up_restores_column_level() {
    // Line 0: "abcdefgh" (len 8)
    // Line 1: "xy" (len 2)
    // Line 2: "0123456789" (len 10)
    let b = buf("abcdefgh\nxy\n0123456789");
    let mut c = cur(2, 7);
    let col_level = 7;

    // Move up to short line — clamps to 2
    let col_level = b.move_cursor_up(&mut c, col_level);
    assert_eq!(c, cur(1, 2));
    assert_eq!(col_level, 7);

    // Move up again to long line — restores to 7
    b.move_cursor_up(&mut c, col_level);
    assert_eq!(c, cur(0, 7));
}

#[test]
fn move_up_first_line_goes_to_col0() {
    let b = buf("hello");
    let mut c = cur(0, 3);
    b.move_cursor_up(&mut c, 3);
    assert_eq!(c, cur(0, 0));
}

#[test]
fn move_down_normal() {
    let b = buf("abcde\nfghij");
    let mut c = cur(0, 3);
    b.move_cursor_down(&mut c, 3);
    assert_eq!(c, cur(1, 3));
}

#[test]
fn move_down_last_line_goes_to_end() {
    let b = buf("hello");
    let mut c = cur(0, 2);
    b.move_cursor_down(&mut c, 2);
    assert_eq!(c, cur(0, 5));
}

#[test]
fn move_line_start_end() {
    let b = buf("hello world");
    let mut c = cur(0, 5);
    b.move_cursor_line_start(&mut c);
    assert_eq!(c, cur(0, 0));
    b.move_cursor_line_end(&mut c);
    assert_eq!(c, cur(0, 11));
}

#[test]
fn move_buffer_start_end() {
    let b = buf("abc\ndef\nghi");
    let mut c = cur(1, 2);
    b.move_cursor_buffer_start(&mut c);
    assert_eq!(c, cur(0, 0));
    b.move_cursor_buffer_end(&mut c);
    assert_eq!(c, cur(2, 3));
}

// ── Group 3: Line Info Helpers ───────────────────────────────────────────────

#[test]
fn line_length_normal() {
    let b = buf("hello\nworld");
    assert_eq!(b.get_line_length(0), 5);
    assert_eq!(b.get_line_length(1), 5);
}

#[test]
fn line_length_last_line_no_newline() {
    let b = buf("abc");
    assert_eq!(b.get_line_length(0), 3);
}

#[test]
fn line_length_empty_line() {
    let b = buf("a\n\nb");
    assert_eq!(b.get_line_length(1), 0);
}

#[test]
fn num_lines() {
    assert_eq!(buf("").get_num_lines(), 1);
    assert_eq!(buf("one").get_num_lines(), 1);
    assert_eq!(buf("a\nb\nc").get_num_lines(), 3);
    assert_eq!(buf("a\n").get_num_lines(), 2);
}

#[test]
fn content_range() {
    let b = buf("line0\nline1\nline2\nline3");
    assert_eq!(b.get_content_range(1, 3), "line1\nline2\n");
}

#[test]
fn get_selection_forward_and_backward() {
    let b = buf("hello world");
    let forward = b.get_selection(&sel(0, 2, 0, 7));
    let backward = b.get_selection(&sel(0, 7, 0, 2));
    assert_eq!(forward, "llo w");
    assert_eq!(forward, backward);
}

#[test]
fn indentation_level() {
    let b = buf("    indented\nno indent\n  two");
    assert_eq!(b.get_indentation_level(0), 4);
    assert_eq!(b.get_indentation_level(1), 0);
    assert_eq!(b.get_indentation_level(2), 2);
}

#[test]
fn word_under_cursor_and_range() {
    let b = buf("hello world");
    // Cursor at col 3 in "hello" → word up to cursor is "hel"
    assert_eq!(b.get_word_under_cursor(&cur(0, 3)), "hel");

    let range = b.get_word_range_under_cursor(&cur(0, 3));
    assert_eq!(range.mark, cur(0, 0));
    assert_eq!(range.cursor, cur(0, 3));
}

#[test]
fn byte_index_from_cursor() {
    let b = buf("abc\ndef");
    assert_eq!(b.byte_index_from_cursor(&cur(0, 0)), 0);
    assert_eq!(b.byte_index_from_cursor(&cur(0, 3)), 3);
    assert_eq!(b.byte_index_from_cursor(&cur(1, 0)), 4); // after newline
    assert_eq!(b.byte_index_from_cursor(&cur(1, 2)), 6);
}

// ── Group 4: Undo / Redo ────────────────────────────────────────────────────

#[test]
fn undo_insert() {
    let mut b = buf("hello");
    b.insert_text(" world", &cur(0, 5), &None, true);
    assert_eq!(content(&b), "hello world");

    let c = b.undo(&None).expect("should undo");
    assert_eq!(c, cur(0, 5));
    assert_eq!(content(&b), "hello");
}

#[test]
fn redo_insert() {
    let mut b = buf("hello");
    b.insert_text(" world", &cur(0, 5), &None, true);
    b.undo(&None);
    assert_eq!(content(&b), "hello");

    let c = b.redo(&None).expect("should redo");
    assert_eq!(c, cur(0, 11));
    assert_eq!(content(&b), "hello world");
}

#[test]
fn undo_delete() {
    let mut b = buf("abcdef");
    b.remove_text(&sel(0, 2, 0, 4), &None, true);
    assert_eq!(content(&b), "abef");

    let c = b.undo(&None).expect("should undo delete");
    assert_eq!(c, cur(0, 4));
    assert_eq!(content(&b), "abcdef");
}

#[test]
fn multiple_undos() {
    let mut b = buf("");
    b.insert_text("A", &cur(0, 0), &None, true);
    b.insert_text("B", &cur(0, 1), &None, true);
    b.insert_text("C", &cur(0, 2), &None, true);
    assert_eq!(content(&b), "ABC");

    b.undo(&None);
    b.undo(&None);
    b.undo(&None);
    assert_eq!(content(&b), "");
}

#[test]
fn undo_then_new_edit_truncates_redo() {
    let mut b = buf("");
    b.insert_text("A", &cur(0, 0), &None, true);
    b.insert_text("B", &cur(0, 1), &None, true);
    b.undo(&None); // undo "B"

    b.insert_text("X", &cur(0, 1), &None, true);
    let result = b.redo(&None);
    assert!(result.is_none(), "redo history should be truncated");
    assert_eq!(content(&b), "AX");
}

#[test]
fn undo_on_empty_history() {
    let mut b = buf("hello");
    assert!(b.undo(&None).is_none());
}

// ── Group 5: Text Transformations ───────────────────────────────────────────

#[test]
fn add_indent_single_line() {
    let mut b = buf("hello");
    let s = sel(0, 0, 0, 5);
    let updated = b.add_indentation(&s, 4, &None);
    assert_eq!(content(&b), "    hello");
    assert_eq!(updated.mark.column, 4);
    assert_eq!(updated.cursor.column, 9);
}

#[test]
fn add_indent_multi_line() {
    let mut b = buf("aaa\nbbb\nccc");
    let s = sel(0, 0, 2, 3);
    b.add_indentation(&s, 2, &None);
    assert_eq!(content(&b), "  aaa\n  bbb\n  ccc");
}

#[test]
fn remove_indent_present() {
    let mut b = buf("    hello");
    let s = sel(0, 4, 0, 9);
    let updated = b.remove_indentation(&s, 4, &None);
    assert_eq!(content(&b), "hello");
    assert_eq!(updated.mark.column, 0);
    assert_eq!(updated.cursor.column, 5);
}

#[test]
fn remove_indent_not_present() {
    let mut b = buf("hello");
    let s = sel(0, 0, 0, 5);
    let updated = b.remove_indentation(&s, 4, &None);
    assert_eq!(content(&b), "hello");
    assert_eq!(updated, s);
}

#[test]
fn toggle_comment_adds() {
    let mut b = buf("hello\nworld");
    let s = sel(0, 0, 1, 5);
    let updated = b.toggle_comment(&s, "// ".to_string(), &None);
    assert_eq!(content(&b), "// hello\n// world");
    assert_eq!(updated.mark.column, 3);
    assert_eq!(updated.cursor.column, 8);
}

#[test]
fn toggle_comment_removes() {
    let mut b = buf("// hello\n// world");
    let s = sel(0, 3, 1, 8);
    let updated = b.toggle_comment(&s, "// ".to_string(), &None);
    assert_eq!(content(&b), "hello\nworld");
    assert_eq!(updated.mark.column, 0);
    assert_eq!(updated.cursor.column, 5);
}

#[test]
fn toggle_comment_with_indent() {
    let mut b = buf("    hello\n    world");
    let s = sel(0, 4, 1, 9);
    b.toggle_comment(&s, "// ".to_string(), &None);
    assert_eq!(content(&b), "    // hello\n    // world");
}

#[test]
fn toggle_comment_empty_token() {
    let mut b = buf("hello");
    let s = sel(0, 0, 0, 5);
    let updated = b.toggle_comment(&s, "".to_string(), &None);
    assert_eq!(content(&b), "hello");
    assert_eq!(updated, s);
}

#[test]
fn select_line_mid_buffer() {
    let b = buf("aaa\nbbb\nccc");
    let s = sel(1, 1, 1, 2);
    let updated = b.select_line(&s);
    assert_eq!(updated.mark.column, 0);
    assert_eq!(updated.cursor, cur(2, 0));
}

#[test]
fn select_line_last_line() {
    let b = buf("aaa\nbbb");
    let s = sel(1, 0, 1, 1);
    let updated = b.select_line(&s);
    assert_eq!(updated.mark.column, 0);
    assert_eq!(updated.cursor, cur(1, 3));
}

#[test]
fn select_word_alphanumeric() {
    let b = buf("hello world");
    let s = sel(0, 0, 0, 0);
    let updated = b.select_word(&s);
    assert_eq!(updated.cursor, cur(0, 5));
}

// ── Group 6: get_visible_lines ──────────────────────────────────────────────

fn params(rows: usize, cols: usize) -> VisibleLineParams {
    VisibleLineParams {
        viewport_rows: rows,
        viewport_columns: cols,
        eol_sequence: "\n".to_string(),
    }
}

fn no_sel() -> Selection {
    sel(0, 0, 0, 0)
}

/// Collect all text from highlighted lines into a single string (lines joined by \n).
fn flatten_text(lines: &HighlightedText) -> String {
    lines
        .iter()
        .map(|line| {
            line.iter()
                .map(|(text, _)| text.as_str())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Check that a specific attribute is present on the span containing `needle`.
fn has_attr_on(lines: &HighlightedText, needle: &str, attr: TextAttributes) -> bool {
    for line in lines {
        for (text, attrs) in line {
            if text.contains(needle) && attrs.contains(attr) {
                return true;
            }
        }
    }
    false
}

/// Check that a specific attribute is present at a given (row, col) in the output.
fn attr_at(lines: &HighlightedText, row: usize, col: usize, attr: TextAttributes) -> bool {
    if let Some(line) = lines.get(row) {
        let mut offset = 0;
        for (text, attrs) in line {
            let len = text.chars().count();
            if col >= offset && col < offset + len {
                return attrs.contains(attr);
            }
            offset += len;
        }
    }
    false
}

// ── 6.1: Basic output & text content ────────────────────────────────────────

#[test]
fn visible_lines_single_short_line() {
    let mut b = buf("hello");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (lines, rel_cur, gutter) =
        b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(lines.len(), 1);
    assert_eq!(gutter.len(), 1);
    assert_eq!(flatten_text(&lines), "hello");
    assert_eq!(rel_cur, cur(0, 0));
}

#[test]
fn visible_lines_multiple_lines() {
    let mut b = buf("aaa\nbbb\nccc");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (lines, _, gutter) =
        b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(lines.len(), 3);
    assert_eq!(gutter.len(), 3);
    assert_eq!(flatten_text(&lines), "aaa\nbbb\nccc");
}

#[test]
fn visible_lines_empty_buffer() {
    let mut b = buf("");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (lines, rel_cur, gutter) =
        b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    // Empty buffer has one line with an empty-string cursor placeholder
    assert_eq!(lines.len(), 1);
    assert_eq!(gutter.len(), 1);
    assert_eq!(rel_cur, cur(0, 0));
}

#[test]
fn visible_lines_single_char() {
    let mut b = buf("x");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (lines, _, _) = b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(flatten_text(&lines), "x");
}

// ── 6.2: Gutter info correctness ───────────────────────────────────────────

#[test]
fn gutter_info_no_wrapping() {
    let mut b = buf("abc\ndef");
    let mut scroll = cur(0, 0);
    let p = params(10, 20); // 17 effective cols, no wrapping needed
    let (_, _, gutter) = b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(gutter.len(), 2);

    assert_eq!(gutter[0].start, cur(0, 0));
    assert_eq!(gutter[0].end, 3);
    assert!(!gutter[0].wrapped);
    assert!(gutter[0].wrap_end);

    assert_eq!(gutter[1].start, cur(1, 0));
    assert_eq!(gutter[1].end, 3);
    assert!(!gutter[1].wrapped);
    assert!(gutter[1].wrap_end);
}

#[test]
fn gutter_info_empty_line() {
    let mut b = buf("abc\n\ndef");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (_, _, gutter) = b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(gutter.len(), 3);

    // Middle empty line
    assert_eq!(gutter[1].start, cur(1, 0));
    assert_eq!(gutter[1].end, 0);
    assert!(!gutter[1].wrapped);
    assert!(gutter[1].wrap_end);
}

#[test]
fn gutter_info_byte_offsets() {
    // "abc\ndef" — line 0 starts at byte 0, line 1 starts at byte 4
    let mut b = buf("abc\ndef");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (_, _, gutter) = b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(gutter[0].start_byte, 0);
    assert_eq!(gutter[0].end_byte, 4); // includes \n
    assert_eq!(gutter[1].start_byte, 4);
    assert_eq!(gutter[1].end_byte, 8); // "def" = 3 chars + forced eol_len=1
}

// ── 6.3: Line wrapping ─────────────────────────────────────────────────────

#[test]
fn line_wrapping_splits_long_line() {
    // viewport_columns=13 → max_characters = 13 - 3 = 10
    // "abcdefghijklmno" is 15 chars → wraps into [0..10) and [10..15)
    let mut b = buf("abcdefghijklmno");
    let mut scroll = cur(0, 0);
    let p = params(10, 13);
    let (lines, _, gutter) =
        b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(gutter.len(), 2);

    // First wrapped segment
    assert_eq!(gutter[0].start, cur(0, 0));
    assert_eq!(gutter[0].end, 10);
    assert!(!gutter[0].wrapped);
    assert!(!gutter[0].wrap_end);

    // Second wrapped segment
    assert_eq!(gutter[1].start, cur(0, 10));
    assert_eq!(gutter[1].end, 15);
    assert!(gutter[1].wrapped);
    assert!(gutter[1].wrap_end);

    assert_eq!(lines.len(), 2);
    assert_eq!(flatten_text(&lines), "abcdefghij\nklmno");
}

#[test]
fn line_wrapping_exact_fit_no_wrap() {
    // viewport_columns=13 → max_characters = 10
    // "abcdefghij" is exactly 10 chars → no wrap
    let mut b = buf("abcdefghij");
    let mut scroll = cur(0, 0);
    let p = params(10, 13);
    let (_, _, gutter) = b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(gutter.len(), 1);
    assert!(!gutter[0].wrapped);
    assert!(gutter[0].wrap_end);
}

#[test]
fn line_wrapping_three_segments() {
    // viewport_columns=8 → max_characters = 5
    // "abcdefghijklm" is 13 chars → 3 segments: [0..5), [5..10), [10..13)
    let mut b = buf("abcdefghijklm");
    let mut scroll = cur(0, 0);
    let p = params(10, 8);
    let (lines, _, gutter) =
        b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(gutter.len(), 3);
    assert!(!gutter[0].wrapped);
    assert!(!gutter[0].wrap_end);
    assert!(gutter[1].wrapped);
    assert!(!gutter[1].wrap_end);
    assert!(gutter[2].wrapped);
    assert!(gutter[2].wrap_end);
    assert_eq!(flatten_text(&lines), "abcde\nfghij\nklm");
}

// ── 6.4: Cursor positioning ─────────────────────────────────────────────────

#[test]
fn relative_cursor_at_origin() {
    let mut b = buf("hello\nworld");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (_, rel_cur, _) = b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(rel_cur, cur(0, 0));
}

#[test]
fn relative_cursor_mid_buffer() {
    let mut b = buf("aaa\nbbb\nccc");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (_, rel_cur, _) = b.get_visible_lines(&mut scroll, Some(&cur(1, 2)), &no_sel(), &p, vec![]);
    assert_eq!(rel_cur, cur(1, 2));
}

#[test]
fn relative_cursor_on_last_line() {
    let mut b = buf("aaa\nbbb\nccc");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (_, rel_cur, _) = b.get_visible_lines(&mut scroll, Some(&cur(2, 1)), &no_sel(), &p, vec![]);
    assert_eq!(rel_cur, cur(2, 1));
}

#[test]
fn relative_cursor_on_wrapped_line() {
    // viewport_columns=13 → max_characters = 10
    // "abcdefghijklmno" wraps: row 0=[0..10), row 1=[10..15)
    // cursor at (0, 12) → in wrapped segment → relative row 1, col 2
    let mut b = buf("abcdefghijklmno");
    let mut scroll = cur(0, 0);
    let p = params(10, 13);
    let (_, rel_cur, _) =
        b.get_visible_lines(&mut scroll, Some(&cur(0, 12)), &no_sel(), &p, vec![]);
    assert_eq!(rel_cur, cur(1, 2));
}

#[test]
fn cursor_at_end_of_line() {
    let mut b = buf("hello\nworld");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (_, rel_cur, _) = b.get_visible_lines(&mut scroll, Some(&cur(0, 5)), &no_sel(), &p, vec![]);
    assert_eq!(rel_cur, cur(0, 5));
}

// ── 6.5: Scroll adjustment ─────────────────────────────────────────────────

#[test]
fn scroll_adjusts_when_cursor_below_viewport() {
    let mut b = buf("line0\nline1\nline2\nline3\nline4");
    let mut scroll = cur(0, 0);
    let p = params(2, 20); // only 2 rows visible
    let (lines, rel_cur, _) =
        b.get_visible_lines(&mut scroll, Some(&cur(3, 0)), &no_sel(), &p, vec![]);
    // Scroll should adjust so cursor is visible
    assert!(scroll.row > 0);
    assert_eq!(lines.len(), 2);
    // Cursor should be within viewport
    assert!(rel_cur.row < 2);
}

#[test]
fn scroll_adjusts_when_cursor_above_viewport() {
    let mut b = buf("line0\nline1\nline2\nline3\nline4");
    let mut scroll = cur(3, 0);
    let p = params(2, 20);
    let (lines, rel_cur, _) =
        b.get_visible_lines(&mut scroll, Some(&cur(1, 0)), &no_sel(), &p, vec![]);
    assert!(scroll.row <= 1);
    assert_eq!(lines.len(), 2);
    assert!(rel_cur.row < 2);
}

#[test]
fn scroll_row_updated_to_match_viewport_start() {
    let mut b = buf("aaa\nbbb\nccc\nddd");
    let mut scroll = cur(0, 0);
    let p = params(2, 20);
    let (_, _, _) = b.get_visible_lines(&mut scroll, Some(&cur(2, 0)), &no_sel(), &p, vec![]);
    // After rendering with cursor on line 2 and viewport of 2, scroll should advance
    assert!(scroll.row <= 2);
}

// ── 6.6: Viewport clipping ─────────────────────────────────────────────────

#[test]
fn viewport_clips_to_row_count() {
    let mut b = buf("line0\nline1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9");
    let mut scroll = cur(0, 0);
    let p = params(3, 20);
    let (lines, _, gutter) =
        b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(lines.len(), 3);
    assert_eq!(gutter.len(), 3);
}

#[test]
fn viewport_larger_than_content() {
    let mut b = buf("only\ntwo");
    let mut scroll = cur(0, 0);
    let p = params(20, 40);
    let (lines, _, gutter) =
        b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(lines.len(), 2);
    assert_eq!(gutter.len(), 2);
}

#[test]
fn viewport_of_one_row() {
    let mut b = buf("aaa\nbbb\nccc");
    let mut scroll = cur(0, 0);
    let p = params(1, 20);
    let (lines, _, _) = b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(lines.len(), 1);
}

// ── 6.7: Selection attributes ───────────────────────────────────────────────

#[test]
fn selection_marks_selected_text() {
    let mut b = buf("hello world");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let selection = sel(0, 2, 0, 7);
    let (lines, _, _) = b.get_visible_lines(&mut scroll, Some(&cur(0, 7)), &selection, &p, vec![]);
    // "llo w" (bytes 2..7) should have SELECT attribute
    assert!(has_attr_on(&lines, "llo", TextAttributes::SELECT));
}

#[test]
fn no_selection_when_mark_equals_cursor() {
    let mut b = buf("hello");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let selection = sel(0, 3, 0, 3); // zero-width selection
    let (lines, _, _) = b.get_visible_lines(&mut scroll, Some(&cur(0, 3)), &selection, &p, vec![]);
    // No span should have SELECT (except possibly merged with CURSOR)
    let any_select = lines.iter().any(|line| {
        line.iter()
            .any(|(_, attrs)| attrs.contains(TextAttributes::SELECT))
    });
    assert!(!any_select);
}

#[test]
fn selection_across_multiple_lines() {
    let mut b = buf("aaa\nbbb\nccc");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let selection = sel(0, 1, 2, 1);
    let (lines, _, _) = b.get_visible_lines(&mut scroll, Some(&cur(2, 1)), &selection, &p, vec![]);
    // Parts of all three lines should have SELECT
    assert!(has_attr_on(&lines, "aa", TextAttributes::SELECT));
    assert!(has_attr_on(&lines, "bbb", TextAttributes::SELECT));
}

#[test]
fn backward_selection_same_as_forward() {
    let mut b = buf("hello world");
    let mut scroll_fwd = cur(0, 0);
    let mut scroll_bwd = cur(0, 0);
    let p = params(10, 20);

    let sel_fwd = sel(0, 2, 0, 7);
    let (lines_fwd, _, _) =
        b.get_visible_lines(&mut scroll_fwd, Some(&cur(0, 7)), &sel_fwd, &p, vec![]);

    let sel_bwd = sel(0, 7, 0, 2);
    let (lines_bwd, _, _) =
        b.get_visible_lines(&mut scroll_bwd, Some(&cur(0, 2)), &sel_bwd, &p, vec![]);

    // Both should mark columns 2..7 with SELECT (cursor position differs,
    // which may split spans differently, so check per-column)
    for col in 2..7 {
        assert!(
            attr_at(&lines_fwd, 0, col, TextAttributes::SELECT),
            "forward: col {} missing SELECT",
            col,
        );
        assert!(
            attr_at(&lines_bwd, 0, col, TextAttributes::SELECT),
            "backward: col {} missing SELECT",
            col,
        );
    }
}

// ── 6.8: Cursor attribute ───────────────────────────────────────────────────

#[test]
fn cursor_attribute_on_correct_position() {
    let mut b = buf("hello");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (lines, _, _) = b.get_visible_lines(&mut scroll, Some(&cur(0, 2)), &no_sel(), &p, vec![]);
    // The character at col 2 ('l') should have CURSOR attribute
    assert!(attr_at(&lines, 0, 2, TextAttributes::CURSOR));
    // Character at col 0 should not have CURSOR
    assert!(!attr_at(&lines, 0, 0, TextAttributes::CURSOR));
}

#[test]
fn cursor_at_eol_gets_space_placeholder() {
    let mut b = buf("hi");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (lines, _, _) = b.get_visible_lines(
        &mut scroll,
        Some(&cur(0, 2)), // past last char
        &no_sel(),
        &p,
        vec![],
    );
    // Should have a space with CURSOR attribute at the end
    let has_cursor_space = lines.iter().any(|line| {
        line.iter()
            .any(|(text, attrs)| text == " " && attrs.contains(TextAttributes::CURSOR))
    });
    assert!(has_cursor_space);
}

#[test]
fn cursor_on_empty_line() {
    let mut b = buf("abc\n\ndef");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (lines, rel_cur, _) =
        b.get_visible_lines(&mut scroll, Some(&cur(1, 0)), &no_sel(), &p, vec![]);
    assert_eq!(rel_cur, cur(1, 0));
    // The empty line should have a space with CURSOR
    let has_cursor_space = lines[1]
        .iter()
        .any(|(text, attrs)| text == " " && attrs.contains(TextAttributes::CURSOR));
    assert!(has_cursor_space);
}

// ── 6.9: No cursor (free scroll / special) ─────────────────────────────────

#[test]
fn no_cursor_returns_zero_relative() {
    let mut b = buf("aaa\nbbb\nccc");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (lines, rel_cur, _) = b.get_visible_lines(&mut scroll, None, &no_sel(), &p, vec![]);
    assert_eq!(rel_cur, cur(0, 0));
    assert_eq!(lines.len(), 3);
}

#[test]
fn special_buffer_ignores_cursor_tracking() {
    let mut b = RopeBuffer::new("aaa\nbbb\nccc".to_string(), None, ".", true);
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (_, rel_cur, _) = b.get_visible_lines(&mut scroll, Some(&cur(2, 0)), &no_sel(), &p, vec![]);
    // Special buffer should not track cursor for scroll
    assert_eq!(rel_cur, cur(0, 0));
}

// ── 6.10: Extra segments (diagnostics) ──────────────────────────────────────

#[test]
fn extra_segments_applied_to_output() {
    let mut b = buf("hello world");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let diagnostics = vec![Range {
        start: 6, // "world" starts at byte 6
        end: 10,
        attributes: TextAttributes::DIAG_ERROR,
    }];
    let (lines, _, _) =
        b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, diagnostics);
    assert!(has_attr_on(&lines, "world", TextAttributes::DIAG_ERROR));
    assert!(!has_attr_on(&lines, "hello", TextAttributes::DIAG_ERROR));
}

#[test]
fn extra_segments_merge_with_selection() {
    let mut b = buf("hello world");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let selection = sel(0, 4, 0, 9);
    let diagnostics = vec![Range {
        start: 6,
        end: 10,
        attributes: TextAttributes::DIAG_WARNING,
    }];
    let (lines, _, _) =
        b.get_visible_lines(&mut scroll, Some(&cur(0, 9)), &selection, &p, diagnostics);
    // "worl" (bytes 6..9) should have both SELECT and DIAG_WARNING
    let has_both = lines.iter().any(|line| {
        line.iter().any(|(_, attrs)| {
            attrs.contains(TextAttributes::SELECT) && attrs.contains(TextAttributes::DIAG_WARNING)
        })
    });
    assert!(has_both);
}

#[test]
fn multiple_diagnostic_segments() {
    let mut b = buf("aaa bbb ccc");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let diagnostics = vec![
        Range {
            start: 0,
            end: 2,
            attributes: TextAttributes::DIAG_ERROR,
        },
        Range {
            start: 8,
            end: 10,
            attributes: TextAttributes::DIAG_HINT,
        },
    ];
    let (lines, _, _) =
        b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, diagnostics);
    // Check per-column: cols 0-2 should have DIAG_ERROR, cols 8-10 should have DIAG_HINT,
    // col 4 (in "bbb") should have neither
    for col in 0..=2 {
        assert!(
            attr_at(&lines, 0, col, TextAttributes::DIAG_ERROR),
            "col {} missing DIAG_ERROR",
            col,
        );
    }
    for col in 8..=10 {
        assert!(
            attr_at(&lines, 0, col, TextAttributes::DIAG_HINT),
            "col {} missing DIAG_HINT",
            col,
        );
    }
    assert!(!attr_at(&lines, 0, 4, TextAttributes::DIAG_ERROR));
    assert!(!attr_at(&lines, 0, 4, TextAttributes::DIAG_HINT));
}

// ── 6.11: VISIBLE attribute ─────────────────────────────────────────────────

#[test]
fn all_output_spans_have_visible() {
    let mut b = buf("hello\nworld\nfoo");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (lines, _, _) = b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    for (row_idx, line) in lines.iter().enumerate() {
        for (text, attrs) in line {
            assert!(
                attrs.contains(TextAttributes::VISIBLE),
                "span '{}' on row {} missing VISIBLE attribute",
                text,
                row_idx,
            );
        }
    }
}

// ── 6.12: Trailing newline / last line edge cases ───────────────────────────

#[test]
fn trailing_newline_adds_empty_line() {
    let mut b = buf("hello\n");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (_, _, gutter) = b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    // "hello\n" has 2 lines: "hello" and ""
    assert_eq!(gutter.len(), 2);
    assert_eq!(gutter[1].start, cur(1, 0));
    assert_eq!(gutter[1].end, 0);
}

#[test]
fn last_line_no_trailing_newline() {
    let mut b = buf("abc\ndef");
    let mut scroll = cur(0, 0);
    let p = params(10, 20);
    let (lines, _, _) = b.get_visible_lines(
        &mut scroll,
        Some(&cur(1, 3)), // cursor at end of "def"
        &no_sel(),
        &p,
        vec![],
    );
    let text = flatten_text(&lines);
    assert!(text.contains("def"));
}

// ── 6.13: Wrapping with multiple buffer lines ───────────────────────────────

#[test]
fn wrapping_interleaved_with_short_lines() {
    // viewport_columns=8 → max_characters = 5
    // line 0: "abcdefgh" (8 chars → wraps to 2 gutter rows)
    // line 1: "xy" (2 chars → 1 gutter row)
    let mut b = buf("abcdefgh\nxy");
    let mut scroll = cur(0, 0);
    let p = params(10, 8);
    let (lines, _, gutter) =
        b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(gutter.len(), 3);
    assert_eq!(gutter[0].start, cur(0, 0)); // first wrap segment
    assert_eq!(gutter[1].start, cur(0, 5)); // second wrap segment
    assert_eq!(gutter[2].start, cur(1, 0)); // "xy"
    assert_eq!(lines.len(), 3);
    assert_eq!(flatten_text(&lines), "abcde\nfgh\nxy");
}

// ── 6.14: Viewport minimum column width ─────────────────────────────────────

#[test]
fn very_narrow_viewport_has_minimum_one_char() {
    // viewport_columns=3 → max_characters = max(3-3, 1) = 1 (saturating_sub then max(1))
    // Actually: 3.saturating_sub(3).max(1) = 0.max(1) = 1
    let mut b = buf("abc");
    let mut scroll = cur(0, 0);
    let p = params(10, 3);
    let (lines, _, gutter) =
        b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    // Each character should be on its own wrapped line
    assert_eq!(gutter.len(), 3);
    assert_eq!(lines.len(), 3);
}

#[test]
fn viewport_columns_less_than_three() {
    // viewport_columns=1 → max_characters = max(0, 1) = 1
    let mut b = buf("hi");
    let mut scroll = cur(0, 0);
    let p = params(10, 1);
    let (lines, _, gutter) =
        b.get_visible_lines(&mut scroll, Some(&cur(0, 0)), &no_sel(), &p, vec![]);
    assert_eq!(gutter.len(), 2); // "h" and "i" each on their own line
    assert_eq!(lines.len(), 2);
}

// ── 6.15: Scroll position output ────────────────────────────────────────────

#[test]
fn scroll_updated_to_reflect_viewport_start_line() {
    let mut b = buf("aaa\nbbb\nccc\nddd\neee");
    let mut scroll = cur(0, 0);
    let p = params(2, 20);
    b.get_visible_lines(&mut scroll, Some(&cur(3, 0)), &no_sel(), &p, vec![]);
    // With cursor on line 3 and viewport of 2, scroll should be at least line 2
    assert!(scroll.row >= 2);
}

// ── 6.16: Idempotency ──────────────────────────────────────────────────────

#[test]
fn calling_twice_with_same_params_gives_same_result() {
    let mut b = buf("hello\nworld\nfoo\nbar");
    let mut scroll1 = cur(0, 0);
    let mut scroll2 = cur(0, 0);
    let p = params(3, 20);
    let cursor = cur(1, 2);
    let selection = no_sel();

    let (lines1, rc1, g1) =
        b.get_visible_lines(&mut scroll1, Some(&cursor), &selection, &p, vec![]);
    let (lines2, rc2, g2) =
        b.get_visible_lines(&mut scroll2, Some(&cursor), &selection, &p, vec![]);

    assert_eq!(flatten_text(&lines1), flatten_text(&lines2));
    assert_eq!(rc1, rc2);
    assert_eq!(g1, g2);
    assert_eq!(scroll1, scroll2);
}
