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
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use uuid::Uuid;

#[derive(Parser)]
pub struct CLIArgs {
    /// Directory to store audio recordings and llm data
    #[arg(required = true, value_name = "FOLDER")]
    app_dir: std::path::PathBuf,

    /// Port number to listen on
    #[arg(default_value_t = 4123)]
    port: u16,
}

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
    recordings_dir: std::path::PathBuf,
    chat_dir: std::path::PathBuf,
    chat_messages: Vec<OllamaChatMessage>,
    chat_options: ChatOptions,
    chat_id: String,
}

pub struct ChatOptions {
    model_name: String,
    temperature: f32,
    seed: u32,
}

pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

#[tokio::main]
async fn main() {
    let cli_args = CLIArgs::parse();
    let recordings_dir = cli_args.app_dir.join("Recordings");
    let chat_dir = cli_args.app_dir.join("Chat");

    let server_state = Arc::new(Mutex::new(ServerState {
        recordings_dir,
        chat_dir,
        chat_messages: vec![],
        chat_options: ChatOptions {
            model_name: "gemma3:1b".into(),
            temperature: 0.5,
            seed: 42,
        },
        chat_id: generate_uuid(),
    }));

    let app = Router::new()
        .route("/", get(info))
        .route("/transcribe", post(transcribe))
        .route("/ws", any(websocket_handler))
        .with_state(server_state)
        .layer(DefaultBodyLimit::disable());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", cli_args.port))
        .await
        .unwrap();
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
            .recordings_dir
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

                let text = ollama_chat(text, state.clone()).await;

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
                sender.send(Message::text(text)).await.unwrap();
            }
        }
    });
}

async fn ollama_chat(prompt: &str, state: Arc<Mutex<ServerState>>) -> String {
    state.lock().unwrap().chat_messages.push(OllamaChatMessage {
        role: "user".into(),
        content: prompt.to_string(),
    });

    let request = OllamaChat {
        model: state.lock().unwrap().chat_options.model_name.clone(),
        messages: state.lock().unwrap().chat_messages.clone(),
        stream: true,
        options: serde_json::json!({
            "seed": state.lock().unwrap().chat_options.seed,
            "temperature": state.lock().unwrap().chat_options.temperature,
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
        content: message_content.clone(),
    });

    let messages = serde_json::to_string_pretty(&state.lock().unwrap().chat_messages).unwrap();
    let save_path = state
        .lock()
        .unwrap()
        .chat_dir
        .join(format!("{}.json", state.lock().unwrap().chat_id));
    let mut f = std::fs::File::create(save_path).unwrap();
    f.write_all(messages.as_bytes()).unwrap();

    message_content
}
