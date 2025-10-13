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
        self.send_action_request(Action::Log(message)).await;
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
