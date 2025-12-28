use clap::Parser;
use rift_core::cli::CLIArgs;

pub mod app;

fn main() -> anyhow::Result<()> {
    rift_core::logging::initialize_tracing();

    tracing::info!("Rift session starting (tui)");

    let cli_args = CLIArgs::parse();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut app = app::App::new(rt, cli_args);

    let mut terminal = ratatui::init();
    terminal.clear()?;
    app.run(terminal)?;
    ratatui::restore();

    tracing::info!("Rift session exiting (tui)");

    Ok(())
}
