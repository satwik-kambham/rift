pub mod app;

fn main() -> anyhow::Result<()> {
    let mut tui_app = app::App::new();
    tui_app.run();
    Ok(())
}
