use clap::Parser;

pub mod app;
pub mod cli;

fn main() -> std::io::Result<()> {
    let args = crate::cli::CLI::parse();
    println!("{}", args.path);
    let mut terminal = crate::app::init()?;
    let mut editor = crate::app::Editor::new()?;
    editor.run(&mut terminal)?;
    crate::app::restore()?;
    Ok(())
}
