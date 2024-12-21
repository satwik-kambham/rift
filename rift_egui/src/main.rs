// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod app;
pub mod command_dispatcher;
pub mod components;

fn main() -> eframe::Result {
    let file_appender = tracing_appender::rolling::never("logs", "rift.log");
    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_level(true)
        .init();
    let native_options = eframe::NativeOptions::default();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let mut app = app::App::new(rt);
    eframe::run_simple_native("Rift", native_options, move |ctx, _frame| {
        app.draw(ctx);
    })
}
