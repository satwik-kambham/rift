use std::io::Write;
use std::sync::{Arc, Mutex};

use axum::{
    Router,
    extract::{
        DefaultBodyLimit, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::{any, get, post},
};
use futures_util::{SinkExt, StreamExt};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct OllamaChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(serde::Serialize)]
/// Request content for ollama generate api
pub struct OllamaChat {
    pub model: String,
    pub messages: Vec<OllamaChatMessage>,
    pub stream: bool,
    pub options: serde_json::Value,
}

struct ServerState {
    recordings_path: std::path::PathBuf,
    chat_messages: Vec<OllamaChatMessage>,
}

#[tokio::main]
async fn main() {
    let recordings_path = std::path::PathBuf::from("/home/satwik/Documents/Recordings/");

    let server_state = Arc::new(Mutex::new(ServerState {
        recordings_path,
        chat_messages: vec![],
    }));

    let app = Router::new()
        .route("/", get(info))
        .route("/transcribe", post(transcribe))
        .route("/chat", post(ollama_chat))
        .route("/ws", any(websocket_handler))
        .with_state(server_state)
        .layer(DefaultBodyLimit::disable());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4123").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn info() -> String {
    "Rift AI".to_string()
}

async fn transcribe(
    State(state): State<Arc<Mutex<ServerState>>>,
    mut multipart: axum::extract::Multipart,
) -> String {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_path = state
            .lock()
            .unwrap()
            .recordings_path
            .join(field.file_name().unwrap());
        let mut file = std::fs::File::create(&file_path).unwrap();
        let data = field.bytes().await.unwrap();
        file.write_all(&data).unwrap();

        let client = reqwest::Client::new();
        let form = reqwest::multipart::Form::new()
            .file("file", file_path)
            .await
            .unwrap();
        let response = client
            .post("http://localhost:5000/transcribe")
            .multipart(form)
            .send()
            .await
            .unwrap();

        let transcript = response.text().await.unwrap();

        return transcript;
    }

    String::new()
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<Mutex<ServerState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(stream: WebSocket, state: Arc<Mutex<ServerState>>) {
    let (mut sender, mut receiver) = stream.split();

    tokio::spawn(async move {
        while let Some(Ok(message)) = &receiver.next().await {
            if let Message::Text(text) = message {
                let text = text.as_str();

                let body = serde_json::json!({
                    "text": text,
                });
                let client = reqwest::Client::new();
                let response = client
                    .post("http://localhost:5000/tts")
                    .json(&body)
                    .send()
                    .await
                    .unwrap();
                let tts = response.bytes().await.unwrap();

                sender.send(Message::binary(tts)).await.unwrap();
            }
        }
    });

    state.lock().unwrap().recordings_path = "".into();
}

async fn ollama_chat(State(state): State<Arc<Mutex<ServerState>>>) {
    state.lock().unwrap().chat_messages.push(OllamaChatMessage {
        role: "user".into(),
        content: "What is the current state of the rust ecosystem for ML? Summarize in 50 words"
            .to_string(),
    });

    let request = OllamaChat {
        model: "qwen3:30b-a3b".to_string(),
        messages: state.lock().unwrap().chat_messages.clone(),
        stream: true,
        options: serde_json::json!({
            "seed": 42,
            "temperature": 0.7,
        }),
    };
    let body = serde_json::to_string(&request).unwrap();

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:11434/api/chat")
        .body(body)
        .send()
        .await
        .unwrap();

    let mut stream = response.bytes_stream();

    let mut message_content = String::new();
    while let Some(chunk) = stream.next().await {
        let parsed_chunk = String::from_utf8(chunk.unwrap().to_vec()).unwrap();
        let parsed_chunk: serde_json::Value = serde_json::from_str(&parsed_chunk).unwrap();
        let content = parsed_chunk["message"]["content"].as_str().unwrap();
        message_content.push_str(content);
    }

    state.lock().unwrap().chat_messages.push(OllamaChatMessage {
        role: "assistant".into(),
        content: message_content,
    });
}
