// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use rift_core::cli::CLIArgs;

pub mod app;
pub mod command_dispatcher;
pub mod components;
pub mod fonts;

fn main() -> eframe::Result {
    rift_core::logging::initialize_tracing();

    tracing::info!("Rift session starting (egui)");

    let cli_args = CLIArgs::parse();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut app = app::App::new(rt, cli_args);

    let native_options = eframe::NativeOptions::default();
    let run_result = eframe::run_simple_native("Rift", native_options, move |ctx, _frame| {
        app.draw(ctx);
    });

    tracing::info!("Rift session exiting (egui)");

    run_result
}
