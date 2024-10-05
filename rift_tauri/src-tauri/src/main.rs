// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use rift_core::state::EditorState;
use tauri::Manager;

pub mod commands;

type AppState = Mutex<EditorState>;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(EditorState::new()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::open_file,
            commands::panel_resized,
            commands::get_visible_lines,
            commands::normal_mode,
            commands::insert_mode,
            commands::move_cursor_left,
            commands::move_cursor_right,
            commands::move_cursor_up,
            commands::move_cursor_down,
            commands::insert_text,
            commands::remove_text,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
