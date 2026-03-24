pub mod message;
pub mod server;

fn main() -> anyhow::Result<()> {
    rift_core::logging::initialize_tracing();

    tracing::info!("Rift session starting (server)");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let mut server = server::Server::new(rt.handle().clone());
    server.run(&rt)?;

    tracing::info!("Rift session exiting (server)");

    Ok(())
}
