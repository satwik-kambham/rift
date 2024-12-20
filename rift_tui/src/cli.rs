use std::path::PathBuf;

use clap::Parser;

/// CLI Arguments
#[derive(Parser, Debug)]
pub struct CLIArgs {
    pub path: Option<PathBuf>,
}
