use std::io::Write;
use std::sync::{Arc, Mutex};

use axum::{
    Router,
    extract::{DefaultBodyLimit, State},
    routing::{get, post},
};

struct ServerState {
    recordings_path: std::path::PathBuf,
}

#[tokio::main]
async fn main() {
    let recordings_path = std::path::PathBuf::from("/home/satwik/Documents/Recordings/");

    let server_state = Arc::new(Mutex::new(ServerState { recordings_path }));

    let app = Router::new()
        .route("/", get(info))
        .route("/transcribe", post(transcribe))
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

        let transcript = String::new();

        return transcript;
    }

    String::new()
}
