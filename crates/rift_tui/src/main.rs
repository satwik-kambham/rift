use clap::Parser;
use rift_core::cli::CLIArgs;

pub mod app;

fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    {
        let mut tmp_dir = std::env::temp_dir();
        tmp_dir.push("rift_logs");
        let file_appender = tracing_appender::rolling::never(tmp_dir, "rift.log");
        tracing_subscriber::fmt()
            .with_env_filter("debug,tarpc=error")
            .with_writer(file_appender)
            .with_ansi(false)
            .with_level(true)
            .init();
    }
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
