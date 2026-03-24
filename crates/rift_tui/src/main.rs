mod app;
mod util;

fn main() -> anyhow::Result<()> {
    rift_core::logging::initialize_tracing();

    tracing::info!("Rift session starting (tui)");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let mut app = app::App::new(rt.handle().clone());

    let mut terminal = ratatui::init();
    terminal.clear()?;
    app.run(&rt, terminal)?;
    ratatui::restore();

    tracing::info!("Rift session exiting (tui)");

    Ok(())
}
