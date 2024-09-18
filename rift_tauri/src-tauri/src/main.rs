// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use rift_core::state::EditorState;
use tauri::Manager;

pub mod command;

type AppState = Mutex<EditorState>;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            command::open_file,
            command::panel_resized,
            command::get_visible_lines,
            command::get_visible_lines_wrap
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
