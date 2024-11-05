// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod app;
pub mod command_dispatcher;

#[tokio::main]
async fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    let mut app = app::App::new().await;
    eframe::run_simple_native("Rift", native_options, move |ctx, _frame| {
        app.draw(ctx);
    })
}
