use rift_core::buffer::instance;
use rift_core::buffer::line_buffer::{HighlightedLine, LineBuffer};
use rift_core::io::file_io;
use rift_core::state::Mode;
use tauri::State;

use crate::AppState;

/// Create buffer from file path and add to state
#[tauri::command]
pub fn open_file(state: State<AppState>, path: String) -> Result<u32, String> {
    let mut state = state.lock().unwrap();
    let initial_text = file_io::read_file_content(&path).map_err(|err| err.to_string())?;
    let buffer = LineBuffer::new(initial_text, Some(path));
    Ok(state.add_buffer(buffer))
}

/// Notify changes to editor panel's size or font changes
#[tauri::command]
pub fn panel_resized(state: State<AppState>, visible_lines: usize, characters_per_line: usize) {
    let mut state = state.lock().unwrap();
    state.visible_lines = visible_lines;
    state.max_characters = characters_per_line;
}

/// Get the lines to be displayed for a given buffer with wrapping
#[tauri::command]
pub fn get_visible_lines(
    state: State<AppState>,
    buffer_id: u32,
) -> (
    HighlightedLine,
    instance::Cursor,
    instance::Cursor,
    Vec<instance::GutterInfo>,
) {
    let mut state = state.lock().unwrap();
    let visible_lines = state.visible_lines;
    let max_characters = state.max_characters;
    let (buffer, instance) = state.get_buffer_by_id_mut(buffer_id);
    let (lines, relative_cursor, gutter_info) = buffer.get_visible_lines(
        &mut instance.scroll,
        &instance.cursor,
        visible_lines,
        max_characters,
        "\n".into(),
    );
    (lines, relative_cursor, instance.cursor, gutter_info)
}

/// Switch to normal mode
#[tauri::command]
pub fn normal_mode(state: State<AppState>) {
    let mut state = state.lock().unwrap();
    state.mode = Mode::Normal;
}

/// Switch to insert mode
#[tauri::command]
pub fn insert_mode(state: State<AppState>) {
    let mut state = state.lock().unwrap();
    state.mode = Mode::Insert;
}

// Navigation Commands
// - Page up / page down => go up and down a whole view optionally with some overlap
// - Move cursor up / down => (insert mode) move cursor and also scroll by half a view / page when cursor over bounds
// - Select prev / next character
// - Move left / right (insert mode)
// - Update selection cursor
// - Update selection mark and cursor
// - Select next word

/// Insert mode - Move cursor right
#[tauri::command]
pub fn move_cursor_right(state: State<AppState>, buffer_id: u32) {
    let mut state = state.lock().unwrap();
    let (buffer, instance) = state.get_buffer_by_id_mut(buffer_id);
    buffer.move_cursor_right(&mut instance.cursor);
}

/// Insert mode - Move cursor left
#[tauri::command]
pub fn move_cursor_left(state: State<AppState>, buffer_id: u32) {
    let mut state = state.lock().unwrap();
    let (buffer, instance) = state.get_buffer_by_id_mut(buffer_id);
    buffer.move_cursor_left(&mut instance.cursor);
}

/// Insert mode - Move cursor up
#[tauri::command]
pub fn move_cursor_up(state: State<AppState>, buffer_id: u32) {
    let mut state = state.lock().unwrap();
    let (buffer, instance) = state.get_buffer_by_id_mut(buffer_id);
    instance.column_level = buffer.move_cursor_up(&mut instance.cursor, instance.column_level);
}

/// Insert mode - Move cursor down
#[tauri::command]
pub fn move_cursor_down(state: State<AppState>, buffer_id: u32) {
    let mut state = state.lock().unwrap();
    let (buffer, instance) = state.get_buffer_by_id_mut(buffer_id);
    instance.column_level = buffer.move_cursor_down(&mut instance.cursor, instance.column_level);
}
