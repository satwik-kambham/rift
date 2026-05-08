use rift_io::logging::initialize_tracing;

pub mod app;

fn main() -> anyhow::Result<()> {
    initialize_tracing();

    tracing::info!("Rift session starting (tui)");

    let mut tui_app = app::App::new();
    tui_app.run();

    tracing::info!("Rift session exiting (tui)");

    Ok(())
}
