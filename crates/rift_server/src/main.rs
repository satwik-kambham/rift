use clap::Parser;
use rift_core::cli::CLIArgs;

pub mod server;

fn main() -> anyhow::Result<()> {
    rift_core::logging::initialize_tracing();

    tracing::info!("Rift session starting (server)");

    let cli_args = CLIArgs::parse();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut server = server::Server::new(rt, cli_args);
    server.run();

    tracing::info!("Rift session exiting (server)");

    Ok(())
}
