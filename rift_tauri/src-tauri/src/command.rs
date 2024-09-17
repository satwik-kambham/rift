use rift_core::buffer::line_buffer::LineBuffer;
use rift_core::io::file_io;
use tauri::State;

use crate::AppState;

#[tauri::command]
pub fn open_file(state: State<AppState>, path: String) -> Result<u32, String> {
    let mut state = state.lock().unwrap();
    let initial_text = file_io::read_file_content(&path).map_err(|err| err.to_string())?;
    let buffer = LineBuffer::new(initial_text, Some(path));
    Ok(state.add_buffer(buffer))
}

#[tauri::command]
pub fn panel_resized(state: State<AppState>, visible_lines: usize) {
    let mut state = state.lock().unwrap();
    state.visible_lines = visible_lines;
}

#[tauri::command]
pub fn get_visible_lines(state: State<AppState>, buffer_id: u32) -> Vec<String> {
    let state = state.lock().unwrap();
    let buffer = state.get_buffer_by_id(buffer_id);
    buffer.get_visible_lines(state.visible_lines).to_vec()
}
