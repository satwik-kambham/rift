use std::collections::HashMap;
use std::path::PathBuf;

use tokio::sync::mpsc;

use rsl::RSL;

use crate::{
    actions::{perform_action, Action},
    buffer::instance::Language,
    lsp::client::LSPClientHandle,
    state::EditorState,
};

pub fn start_rsl_interpreter(
    initial_folder: String,
    rpc_client_transport: tarpc::transport::channel::UnboundedChannel<
        tarpc::Response<rift_rpc::RiftRPCResponse>,
        tarpc::ClientMessage<rift_rpc::RiftRPCRequest>,
    >,
) -> mpsc::Sender<String> {
    let (rsl_sender, mut rsl_reciever) = mpsc::channel::<String>(32);

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let mut rsl_interpreter = RSL::new(
            Some(PathBuf::from(&initial_folder)),
            rt.handle().clone(),
            rpc_client_transport,
        );
        while let Some(source) = rsl_reciever.blocking_recv() {
            rsl_interpreter.run(source);
        }
    });

    rsl_sender
}

pub fn initialize_rsl(
    state: &mut EditorState,
    lsp_handles: &mut HashMap<Language, LSPClientHandle>,
) {
    #[cfg(not(debug_assertions))]
    let init_module = include_str!("../modules/init.rsl").to_string();
    #[cfg(debug_assertions)]
    let init_module = std::fs::read_to_string("crates/rift_core/modules/init.rsl").unwrap();
    perform_action(
        Action::RunSource(init_module),
        state,
        lsp_handles,
    );
}
