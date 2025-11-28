use std::{collections::HashMap, net::SocketAddr, time::Duration};

use axum::{
    extract::ConnectInfo,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
};
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc};

use rift_core::{
    actions::{Action, perform_action},
    buffer::instance::Language,
    cli::{CLIArgs, process_cli_args},
    io::file_io::handle_file_event,
    lsp::{client::LSPClientHandle, handle_lsp_messages},
    rsl::initialize_rsl,
    state::EditorState,
};

pub async fn start_axum_server(
    sender_to_ws: broadcast::Sender<Bytes>,
    sender_from_ws: mpsc::Sender<Bytes>,
) {
    let app = axum::Router::new().route(
        "/ws",
        axum::routing::get(move |ws, info| {
            ws_handler(ws, info, sender_to_ws.subscribe(), sender_from_ws.clone())
        }),
    );
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
            .await
            .unwrap();
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    receiver_to_ws: broadcast::Receiver<Bytes>,
    sender_from_ws: mpsc::Sender<Bytes>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr, receiver_to_ws, sender_from_ws))
}

async fn handle_socket(
    socket: WebSocket,
    _addr: SocketAddr,
    mut receiver_to_ws: broadcast::Receiver<Bytes>,
    sender_from_ws: mpsc::Sender<Bytes>,
) {
    let (mut socket_sender, mut socket_receiver) = socket.split();
    tokio::spawn(async move {
        while let Some(Ok(message)) = socket_receiver.next().await {
            match message {
                Message::Binary(bytes) => {
                    sender_from_ws.send(bytes).await.unwrap();
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::spawn(async move {
        while let Ok(bytes) = receiver_to_ws.recv().await {
            socket_sender.send(Message::Binary(bytes)).await.unwrap();
        }
    });
}

pub struct Server {
    pub state: EditorState,
    pub lsp_handles: HashMap<Language, LSPClientHandle>,
    pub sender_to_ws: broadcast::Sender<Bytes>,
    pub receiver_from_ws: mpsc::Receiver<Bytes>,
}

impl Server {
    pub fn new(rt: tokio::runtime::Runtime, cli_args: CLIArgs) -> Self {
        let (sender_to_ws, _) = broadcast::channel::<Bytes>(32);
        let (sender_from_ws, receiver_from_ws) = mpsc::channel::<Bytes>(32);

        rt.block_on(async { start_axum_server(sender_to_ws.clone(), sender_from_ws).await });

        let mut state = EditorState::new(rt);
        let mut lsp_handles = HashMap::new();

        process_cli_args(cli_args, &mut state, &mut lsp_handles);

        initialize_rsl(&mut state, &mut lsp_handles);

        Self {
            state,
            lsp_handles,
            sender_to_ws,
            receiver_from_ws,
        }
    }

    pub fn perform_action(&mut self, action: Action) -> String {
        perform_action(action, &mut self.state, &mut self.lsp_handles).unwrap_or_default()
    }

    pub fn run(&mut self) {
        while !self.state.quit {
            // Run async callbacks
            if let Ok(async_result) = self.state.async_handle.receiver.try_recv() {
                (async_result.callback)(
                    async_result.result,
                    &mut self.state,
                    &mut self.lsp_handles,
                );
                self.state.update_view = true;
            }

            // Run action requests
            while let Ok(action_request) = self.state.event_reciever.try_recv() {
                let result = self.perform_action(action_request.action);
                action_request.response_tx.send(result).unwrap();
                self.state.update_view = true;
            }

            // Handle file watcher events
            if let Ok(file_event_result) = self.state.file_event_receiver.try_recv() {
                handle_file_event(file_event_result, &mut self.state, &mut self.lsp_handles);
                self.state.update_view = true;
            }

            // Handle lsp messages
            handle_lsp_messages(&mut self.state, &mut self.lsp_handles);

            // Handle websocket messages
            if let Ok(_bytes) = self.receiver_from_ws.try_recv() {
                self.state.update_view = true;
            }

            // Update view and send to websocket connection
            if self.state.update_view {
                // self.state.relative_cursor =
                //     update_visible_lines(&mut self.state, visible_lines, max_characters);

                self.state.update_view = false;
            }

            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
