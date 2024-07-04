use clap::Parser;

pub mod app;
pub mod cli;

fn main() -> std::io::Result<()> {
    let args = crate::cli::CLI::parse();
    println!("{}", args.path);
    let editor = crate::app::Editor::new()?;
    editor.render()?;
    crate::app::restore()?;
    Ok(())
}
