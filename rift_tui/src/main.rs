use clap::Parser;

pub mod app;
pub mod cli;

fn main() -> anyhow::Result<()> {
    let cli_args = cli::CLIArgs::parse();
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
