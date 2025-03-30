// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use rift_core::cli::CLIArgs;

pub mod app;
pub mod command_dispatcher;
pub mod components;
pub mod fonts;

fn main() -> eframe::Result {
    let file_appender = tracing_appender::rolling::never("logs", "rift.log");
    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_level(true)
        .init();
    let cli_args = CLIArgs::parse();
    let native_options = eframe::NativeOptions::default();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let mut app = app::App::new(rt, cli_args);
    eframe::run_simple_native("Rift", native_options, move |ctx, _frame| {
        app.draw(ctx);
    })
}
