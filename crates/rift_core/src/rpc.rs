use futures::prelude::*;
use tarpc::server::Channel;

use tokio::sync::mpsc::Sender;

use rift_rpc::RiftRPC;

use crate::actions::Action;

#[derive(Clone)]
pub struct RPCHandle {
    pub sender: Sender<Action>,
}

impl RiftRPC for RPCHandle {
    async fn rlog(self, _context: tarpc::context::Context, message: String) -> () {
        self.sender.send(Action::Log(message)).await.unwrap();
    }
}

pub async fn start_rpc_server(
    event_sender: Sender<Action>,
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
