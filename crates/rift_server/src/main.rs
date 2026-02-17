pub mod server;

fn main() -> anyhow::Result<()> {
    rift_core::logging::initialize_tracing();

    tracing::info!("Rift session starting (server)");

    let mut server = server::Server::new();
    server.run()?;

    tracing::info!("Rift session exiting (server)");

    Ok(())
}
