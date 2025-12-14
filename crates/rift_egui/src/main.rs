// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use rift_core::cli::CLIArgs;
use tracing::info;

pub mod app;
pub mod command_dispatcher;
pub mod components;
pub mod fonts;

fn main() -> eframe::Result {
    let mut tmp_dir = std::env::temp_dir();
    tmp_dir.push("rift_logs");
    let file_appender = tracing_appender::rolling::never(tmp_dir, "rift.log");
    tracing_subscriber::fmt()
        .with_env_filter("debug,tarpc=error")
        .with_writer(file_appender)
        .with_ansi(false)
        .with_level(true)
        .init();

    info!("Rift session starting (egui)");
    let cli_args = CLIArgs::parse();
    let native_options = eframe::NativeOptions::default();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let mut app = app::App::new(rt, cli_args);
    let run_result = eframe::run_simple_native("Rift", native_options, move |ctx, _frame| {
        app.draw(ctx);
    });
    info!("Rift session exiting (egui)");
    run_result
}
