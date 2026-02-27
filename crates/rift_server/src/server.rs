use std::{net::SocketAddr, time::Duration};

use axum::{
    extract::ConnectInfo,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
};
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use serde_json::to_value;
use tokio::sync::{broadcast, mpsc};
use tower_http::services::ServeDir;

use rift_core::{
    actions::{Action, perform_action},
    io::file_io::handle_file_event,
    lsp::handle_lsp_messages,
    rendering::update_visible_lines,
    state::EditorState,
};

use crate::message::{ConnectionStatus, InitializeData, Message as JsonMessage};

#[derive(Clone)]
enum WSMessage {
    Bytes(Bytes),
    Text(String),
}

async fn start_axum_server(
    sender_to_ws: broadcast::Sender<WSMessage>,
    sender_from_ws: mpsc::Sender<WSMessage>,
) {
    let static_files = ServeDir::new("static");
    let app = axum::Router::new()
        .route(
            "/ws",
            axum::routing::get(move |ws, info| {
                ws_handler(ws, info, sender_to_ws.subscribe(), sender_from_ws.clone())
            }),
        )
        .fallback_service(static_files);
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
    receiver_to_ws: broadcast::Receiver<WSMessage>,
    sender_from_ws: mpsc::Sender<WSMessage>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr, receiver_to_ws, sender_from_ws))
}

async fn handle_socket(
    socket: WebSocket,
    _addr: SocketAddr,
    mut receiver_to_ws: broadcast::Receiver<WSMessage>,
    sender_from_ws: mpsc::Sender<WSMessage>,
) {
    let (mut socket_sender, mut socket_receiver) = socket.split();
    tokio::spawn(async move {
        while let Some(Ok(message)) = socket_receiver.next().await {
            match message {
                Message::Text(text) => {
                    sender_from_ws
                        .send(WSMessage::Text(text.to_string()))
                        .await
                        .unwrap();
                }
                Message::Binary(bytes) => {
                    sender_from_ws.send(WSMessage::Bytes(bytes)).await.unwrap();
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::spawn(async move {
        while let Ok(message) = receiver_to_ws.recv().await {
            let message = match message {
                WSMessage::Bytes(bytes) => Message::Binary(bytes),
                WSMessage::Text(text) => Message::Text(text.into()),
            };
            socket_sender.send(message).await.unwrap();
        }
    });
}

pub(crate) struct Server {
    state: EditorState,
    sender_to_ws: broadcast::Sender<WSMessage>,
    receiver_from_ws: mpsc::Receiver<WSMessage>,
    status: ConnectionStatus,
    viewport_rows: usize,
    viewport_columns: usize,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub(crate) fn new() -> Self {
        let (sender_to_ws, _) = broadcast::channel::<WSMessage>(32);
        let (sender_from_ws, receiver_from_ws) = mpsc::channel::<WSMessage>(32);

        let mut state = EditorState::new();
        state.post_initialization();

        state
            .rt
            .block_on(async { start_axum_server(sender_to_ws.clone(), sender_from_ws).await });

        Self {
            state,
            sender_to_ws,
            receiver_from_ws,
            status: ConnectionStatus::Disconnected,
            viewport_rows: 24,
            viewport_columns: 80,
        }
    }

    fn perform_action(&mut self, action: Action) -> String {
        perform_action(action, &mut self.state).unwrap_or_default()
    }

    pub(crate) fn run(&mut self) -> anyhow::Result<()> {
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
            if let Ok(message) = self.receiver_from_ws.try_recv() {
                match message {
                    WSMessage::Text(text) => {
                        if let Ok(msg) = serde_json::from_str::<JsonMessage>(&text) {
                            match msg.method.as_str() {
                                "connected" => {
                                    self.status = ConnectionStatus::Connected;
                                    let initialize_data = InitializeData {
                                        editor_font_size: self.state.preferences.editor_font_size,
                                    };
                                    let response = JsonMessage {
                                        method: "initialize".to_string(),
                                        data: Some(to_value(initialize_data).unwrap()),
                                    };
                                    if let Ok(json) = serde_json::to_string(&response) {
                                        let _ = self.sender_to_ws.send(WSMessage::Text(json));
                                    }
                                }
                                "initialized" => {
                                    self.status = ConnectionStatus::Initialized;
                                    if let Some(data) = msg.data {
                                        if let Some(rows) =
                                            data.get("viewport_rows").and_then(|v| v.as_u64())
                                        {
                                            self.viewport_rows = rows as usize;
                                        }
                                        if let Some(cols) =
                                            data.get("viewport_columns").and_then(|v| v.as_u64())
                                        {
                                            self.viewport_columns = cols as usize;
                                        }
                                    }
                                }
                                "run_action" => {
                                    if let Some(data) = msg.data
                                        && let Some(action_name) = data.as_str()
                                    {
                                        self.perform_action(Action::RunAction(
                                            action_name.to_string(),
                                        ));
                                    }
                                }
                                _ => {
                                    tracing::info!("Unknown method: {}", msg.method);
                                }
                            }
                        }
                    }
                    WSMessage::Bytes(_bytes) => {
                        tracing::info!("Received binary");
                    }
                }
                self.state.update_view = true;
            }

            // Update view and send to websocket connection
            if self.state.update_view {
                self.state.relative_cursor = update_visible_lines(
                    &mut self.state,
                    self.viewport_rows,
                    self.viewport_columns,
                    true,
                );

                if self.status == ConnectionStatus::Initialized {
                    let response = JsonMessage {
                        method: "render".to_string(),
                        data: Some(to_value(&self.state.highlighted_text).unwrap()),
                    };
                    if let Ok(json) = serde_json::to_string(&response) {
                        let _ = self.sender_to_ws.send(WSMessage::Text(json));
                    }
                }

                self.state.update_view = false;
            }
        }
        Ok(())
    }
}
