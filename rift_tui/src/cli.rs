use clap::Parser;

/// Editor CLI
#[derive(Debug, Parser)]
pub struct CLI {
    /// Path to file or folder to open
    pub path: String,
}
