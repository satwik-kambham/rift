use std::io::Write;
use std::sync::{Arc, Mutex};

use axum::{
    Router,
    extract::{DefaultBodyLimit, State},
    routing::{get, post},
};
use candle_core::Device;
use candle_transformers::models::whisper::Config;

pub mod whisper;

struct ServerState {
    recordings_path: std::path::PathBuf,
    decoder: whisper::Decoder,
    config: Config,
    mel_filters: Vec<f32>,
    device: Device,
}

#[tokio::main]
async fn main() {
    let recordings_path = std::path::PathBuf::from("/home/satwik/Documents/Recordings/");
    let config_file = "/home/satwik/Documents/rift/examples/whisper/tiny-en/config.json";
    let model_file = "/home/satwik/Documents/rift/examples/whisper/tiny-en/model.safetensors";
    let tokenizer_file = "/home/satwik/Documents/rift/examples/whisper/tiny-en/tokenizer.json";
    let device = Device::Cpu;

    let (decoder, config, mel_filters) =
        whisper::load_model(config_file, tokenizer_file, model_file, &device);

    let server_state = Arc::new(Mutex::new(ServerState {
        decoder,
        config,
        mel_filters,
        device,
        recordings_path,
    }));

    let app = Router::new()
        .route("/", get(info))
        .route("/transcribe", post(transcribe))
        .with_state(server_state)
        .layer(DefaultBodyLimit::disable());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4123")
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
            .recordings_path
            .join(field.file_name().unwrap());
        let mut file = std::fs::File::create(&file_path).unwrap();
        let data = field.bytes().await.unwrap();
        file.write_all(&data).unwrap();

        let mut state = state.lock().unwrap();
        let mel = whisper::preprocess_input(
            file_path.to_str().unwrap(),
            &state.config,
            &state.device,
            &state.mel_filters,
        );
        let transcription = state.decoder.run(&mel);
        return transcription
            .iter()
            .map(|segment| segment.dr.text.clone())
            .collect::<Vec<_>>()
            .join(" ");
    }

    String::new()
}
