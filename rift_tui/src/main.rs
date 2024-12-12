pub mod app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let mut app = app::App::new().await;
    app.run(terminal).await?;
    ratatui::restore();
    Ok(())
}
