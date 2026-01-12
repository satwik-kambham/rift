// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod app;
pub mod command_dispatcher;
pub mod components;
pub mod fonts;

fn main() -> eframe::Result {
    rift_core::logging::initialize_tracing();

    tracing::info!("Rift session starting (egui)");

    let mut app = app::App::new();

    let native_options = eframe::NativeOptions::default();
    let run_result = eframe::run_ui_native("Rift", native_options, move |ui, _frame| {
        app.draw(ui);
    });

    tracing::info!("Rift session exiting (egui)");

    run_result
}
