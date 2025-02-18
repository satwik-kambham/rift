use clap::Parser;
use rift_core::cli::CLIArgs;

pub mod app;

fn main() -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::never("logs", "rift.log");
    tracing_subscriber::fmt()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_level(true)
        .init();
    let cli_args = CLIArgs::parse();
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let mut app = app::App::new(rt, cli_args);
    app.run(terminal)?;
    ratatui::restore();
    Ok(())
}
