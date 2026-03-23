use std::net::SocketAddr;

use axum::{
    extract::ConnectInfo,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
};
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use serde_json::to_value;
use tokio::sync::{broadcast, mpsc};
use tokio::task::LocalSet;
use tower_http::services::ServeDir;

use petal::Block;
use rift_core::{
    actions::{Action, perform_action},
    audio::{build_temp_path, convert_webm_to_wav, transcribe_wav_file},
    io::file_io::handle_file_event,
    lsp::handle_lsp_message,
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
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
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

impl Server {
    pub(crate) fn new(rt_handle: tokio::runtime::Handle) -> Self {
        let (sender_to_ws, _) = broadcast::channel::<WSMessage>(32);
        let (sender_from_ws, receiver_from_ws) = mpsc::channel::<WSMessage>(32);

        let mut state = EditorState::new(rt_handle.clone());
        state.post_initialization();

        rt_handle.block_on(async { start_axum_server(sender_to_ws.clone(), sender_from_ws).await });

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

    pub(crate) fn run(&mut self, rt: &tokio::runtime::Runtime) -> anyhow::Result<()> {
        let local = LocalSet::new();
        local.block_on(rt, async {
            loop {
                if self.state.quit {
                    break;
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

                tokio::select! {
                    Some(req) = self.state.event_reciever.recv() => {
                        let result = self.perform_action(req.action);
                        req.response_tx.send(result).unwrap();
                        self.state.update_view = true;
                    }
                    Some(async_result) = self.state.async_handle.receiver.recv() => {
                        (async_result.callback)(async_result.result, &mut self.state);
                        self.state.update_view = true;
                    }
                    Some(file_event) = self.state.file_event_receiver.recv() => {
                        handle_file_event(file_event, &mut self.state);
                        self.state.update_view = true;
                    }
                    Some(lsp_msg) = self.state.lsp_message_receiver.recv() => {
                        handle_lsp_message(lsp_msg, &mut self.state);
                        self.state.update_view = true;
                    }
                    Some(message) = self.receiver_from_ws.recv() => {
                        self.handle_ws_message(message);
                        self.state.update_view = true;
                    }
                }
            }
        });
        Ok(())
    }

    fn handle_ws_message(&mut self, message: WSMessage) {
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
                                self.perform_action(Action::RunAction(action_name.to_string()));
                            }
                        }
                        "ping" => {
                            let response = JsonMessage {
                                method: "pong".to_string(),
                                data: msg.data,
                            };
                            if let Ok(json) = serde_json::to_string(&response) {
                                let _ = self.sender_to_ws.send(WSMessage::Text(json));
                            }
                        }
                        _ => {
                            tracing::info!("Unknown method: {}", msg.method);
                        }
                    }
                }
            }
            WSMessage::Bytes(bytes) => {
                let webm_path = build_temp_path("webm");

                if let Err(e) = std::fs::write(&webm_path, &bytes) {
                    tracing::error!("Failed to save audio recording: {}", e);
                    return;
                }
                tracing::info!("Saved audio recording to {}", webm_path.display());

                let wav_path = match convert_webm_to_wav(&webm_path) {
                    Ok(path) => path,
                    Err(e) => {
                        tracing::error!("Failed to convert webm to wav: {}", e);
                        return;
                    }
                };
                tracing::info!("Converted to wav: {}", wav_path.display());

                let wav_data = std::fs::read(&wav_path);
                let note_id = if let Some(ref store) = self.state.note_store
                    && let Ok(wav_bytes) = &wav_data
                {
                    match store.create_note() {
                        Ok(mut note) => {
                            let wav_filename = wav_path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            match store.write_attachment(note.id, &wav_filename, wav_bytes) {
                                Ok(block) => {
                                    note.blocks.push(block);
                                    if let Err(err) = store.save_note(note.clone()) {
                                        tracing::warn!(
                                            %err,
                                            "Failed to save note with audio attachment"
                                        );
                                    }
                                }
                                Err(err) => {
                                    tracing::warn!(
                                        %err, "Failed to write audio attachment"
                                    )
                                }
                            }
                            Some(note)
                        }
                        Err(err) => {
                            tracing::warn!(
                                %err, "Failed to create note for transcription"
                            );
                            None
                        }
                    }
                } else {
                    None
                };

                let transcription = match transcribe_wav_file(wav_path.clone()) {
                    Ok(text) => text,
                    Err(e) => {
                        tracing::error!("Failed to transcribe wav: {}", e);
                        let _ = std::fs::remove_file(wav_path);
                        return;
                    }
                };
                tracing::info!("Transcription: {}", transcription);

                let _ = std::fs::remove_file(wav_path);

                if let Some(ref store) = self.state.note_store
                    && let Some(mut note) = note_id
                {
                    let note_id = note.id;
                    note.blocks.push(Block::Text {
                        label: Some("transcription".to_string()),
                        content: transcription.clone(),
                    });
                    if let Err(err) = store.save_note(note) {
                        tracing::warn!(
                            %err, "Failed to save transcription to note"
                        );
                    }
                    let note_file = store.note_path(note_id).to_string_lossy().to_string();
                    perform_action(Action::OpenFile(note_file), &mut self.state);
                }

                let response = JsonMessage {
                    method: "transcription".to_string(),
                    data: Some(to_value(transcription).unwrap()),
                };
                if let Ok(json) = serde_json::to_string(&response) {
                    let _ = self.sender_to_ws.send(WSMessage::Text(json));
                }
            }
        }
    }
}
