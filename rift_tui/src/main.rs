use clap::Parser;

pub mod app;
pub mod cli;

fn main() -> std::io::Result<()> {
    let args = crate::cli::CLI::parse();
    println!("{}", args.path);
    crate::app::app()
}
