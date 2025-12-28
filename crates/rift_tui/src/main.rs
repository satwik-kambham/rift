pub mod app;

fn main() -> anyhow::Result<()> {
    rift_core::logging::initialize_tracing();

    tracing::info!("Rift session starting (tui)");

    let mut app = app::App::new();

    let mut terminal = ratatui::init();
    terminal.clear()?;
    app.run(terminal)?;
    ratatui::restore();

    tracing::info!("Rift session exiting (tui)");

    Ok(())
}
