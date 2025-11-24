use futures::prelude::*;
use tarpc::server::Channel;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use rift_rpc::RiftRPC;

use crate::actions::Action;

pub struct RPCRequest {
    pub action: Action,
    pub response_tx: oneshot::Sender<String>,
}

#[derive(Clone)]
pub struct RPCHandle {
    pub sender: mpsc::Sender<RPCRequest>,
}

impl RPCHandle {
    async fn send_action_request(&self, action: Action) -> String {
        let (response_tx, response_rx) = oneshot::channel();
        self.sender
            .send(RPCRequest {
                action,
                response_tx,
            })
            .await
            .unwrap();
        response_rx.await.unwrap()
    }
}

impl RiftRPC for RPCHandle {
    async fn rlog(self, _context: tarpc::context::Context, message: String) {
        tracing::info!("{}", message);
        self.send_action_request(Action::Log(message)).await;
    }

    async fn set_active_buffer(self, _context: tarpc::context::Context, id: u32) {
        self.send_action_request(Action::SetActiveBuffer(id)).await;
    }

    async fn register_global_keybind(
        self,
        _context: tarpc::context::Context,
        definition: String,
        function_id: String,
    ) {
        self.send_action_request(Action::RegisterGlobalKeybind(definition, function_id))
            .await;
    }

    async fn create_special_buffer(
        self,
        _context: tarpc::context::Context,
        display_name: String,
    ) -> u32 {
        let buffer_id = self
            .send_action_request(Action::CreateSpecialBuffer(display_name))
            .await;
        buffer_id.parse().unwrap()
    }

    async fn open_file(self, _context: tarpc::context::Context, path: String) {
        self.send_action_request(Action::OpenFile(path)).await;
    }

    async fn register_buffer_keybind(
        self,
        _context: tarpc::context::Context,
        buffer_id: u32,
        definition: String,
        function_id: String,
    ) {
        self.send_action_request(Action::RegisterBufferKeybind(
            buffer_id,
            definition,
            function_id,
        ))
        .await;
    }

    async fn set_buffer_content(
        self,
        _context: tarpc::context::Context,
        buffer_id: u32,
        content: String,
    ) {
        self.send_action_request(Action::SetBufferContent(buffer_id, content))
            .await;
    }

    async fn get_buffer_input(self, _context: tarpc::context::Context, buffer_id: u32) -> String {
        self.send_action_request(Action::GetBufferInput(buffer_id))
            .await
    }

    async fn set_buffer_input(
        self,
        _context: tarpc::context::Context,
        buffer_id: u32,
        input: String,
    ) {
        self.send_action_request(Action::SetBufferInput(buffer_id, input))
            .await;
    }

    async fn register_buffer_input_hook(
        self,
        _context: tarpc::context::Context,
        buffer_id: u32,
        function_id: String,
    ) {
        self.send_action_request(Action::RegisterBufferInputHook(buffer_id, function_id))
            .await;
    }

    async fn get_workspace_dir(self, _context: tarpc::context::Context) -> String {
        self.send_action_request(Action::GetWorkspaceDir).await
    }

    async fn run_action(self, _context: tarpc::context::Context, action: String) -> String {
        self.send_action_request(Action::RunAction(action)).await
    }

    async fn get_active_buffer(self, _context: tarpc::context::Context) -> u32 {
        self.send_action_request(Action::GetActiveBuffer)
            .await
            .parse()
            .unwrap_or_default()
    }
}

pub async fn start_rpc_server(
    event_sender: mpsc::Sender<RPCRequest>,
) -> tarpc::transport::channel::UnboundedChannel<
    tarpc::Response<rift_rpc::RiftRPCResponse>,
    tarpc::ClientMessage<rift_rpc::RiftRPCRequest>,
> {
    let (client_transport, server_transport) = tarpc::transport::channel::unbounded();

    let channel = tarpc::server::BaseChannel::with_defaults(server_transport);
    let server = RPCHandle {
        sender: event_sender,
    };
    tokio::spawn(
        channel
            .execute(server.serve())
            .for_each(|response| async move {
                tokio::spawn(response);
            }),
    );

    client_transport
}
