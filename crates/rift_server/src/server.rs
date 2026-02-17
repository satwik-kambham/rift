use std::{net::SocketAddr, time::Duration};

use axum::{
    extract::ConnectInfo,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
};
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc};

use rift_core::{
    actions::{Action, perform_action},
    io::file_io::handle_file_event,
    lsp::handle_lsp_messages,
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
    pub sender_to_ws: broadcast::Sender<Bytes>,
    pub receiver_from_ws: mpsc::Receiver<Bytes>,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        let (sender_to_ws, _) = broadcast::channel::<Bytes>(32);
        let (sender_from_ws, receiver_from_ws) = mpsc::channel::<Bytes>(32);

        let mut state = EditorState::new();
        state.post_initialization();

        state
            .rt
            .block_on(async { start_axum_server(sender_to_ws.clone(), sender_from_ws).await });

        Self {
            state,
            sender_to_ws,
            receiver_from_ws,
        }
    }

    fn perform_action(&mut self, action: Action) -> String {
        perform_action(action, &mut self.state).unwrap_or_default()
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        while !self.state.quit {
            // Run async callbacks
            if let Ok(async_result) = self.state.async_handle.receiver.try_recv() {
                (async_result.callback)(async_result.result, &mut self.state);
                self.state.update_view = true;
            }

            // Run action requests
            while let Ok(action_request) = self.state.event_reciever.try_recv() {
                let result = self.perform_action(action_request.action);
                action_request.response_tx.send(result).unwrap();
                self.state.update_view = true;
                std::thread::sleep(Duration::from_millis(1));
            }

            // Handle file watcher events
            if let Ok(file_event_result) = self.state.file_event_receiver.try_recv() {
                handle_file_event(file_event_result, &mut self.state);
                self.state.update_view = true;
            }

            // Handle lsp messages
            handle_lsp_messages(&mut self.state);

            // Handle websocket messages
            if let Ok(_bytes) = self.receiver_from_ws.try_recv() {
                self.state.update_view = true;
            }

            // Update view and send to websocket connection
            if self.state.update_view {
                // self.state.relative_cursor =
                //     update_visible_lines(&mut self.state, viewport_rows, viewport_columns);

                self.state.update_view = false;
            }
        }
        Ok(())
    }
}
