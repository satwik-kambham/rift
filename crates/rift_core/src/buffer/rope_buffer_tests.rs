use super::instance::{Cursor, Selection};
use super::rope_buffer::RopeBuffer;

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
