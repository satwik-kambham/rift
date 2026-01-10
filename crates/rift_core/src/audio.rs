use std::{
    fs::File,
    io::BufWriter,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use cpal::{
    SampleFormat, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use tokio::sync::mpsc::Sender;

use crate::{
    concurrent::{AsyncError, AsyncResult},
    state::EditorState,
};

#[derive(Debug, Clone)]
pub struct InputDeviceInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("audio device error: {0}")]
    Device(String),
    #[error("audio configuration error: {0}")]
    Config(String),
    #[error("audio I/O error: {0}")]
    Io(String),
    #[error("wav error: {0}")]
    Wav(String),
    #[error("network error: {0}")]
    Network(String),
}

pub type TranscriptionCallback = fn(Result<String, AudioError>, &mut EditorState);

pub struct TranscriptionHandle {
    stream: Option<Stream>,
    writer: Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>,
    path: PathBuf,
    callback: TranscriptionCallback,
    sender: Sender<AsyncResult>,
    rt_handle: tokio::runtime::Handle,
    stopped: AtomicBool,
}

impl TranscriptionHandle {
    pub fn stop(&mut self) {
        if self.stopped.swap(true, Ordering::SeqCst) {
            return;
        }

        self.stream.take();

        let mut guard = match self.writer.lock() {
            Ok(guard) => guard,
            Err(_) => {
                cleanup_temp_file(&self.path);
                send_transcription_result(
                    Err(AudioError::Io("Failed to lock audio writer".to_string())),
                    self.rt_handle.clone(),
                    self.sender.clone(),
                );
                return;
            }
        };

        let Some(writer) = guard.take() else {
            cleanup_temp_file(&self.path);
            send_transcription_result(
                Err(AudioError::Wav("Audio writer missing".to_string())),
                self.rt_handle.clone(),
                self.sender.clone(),
            );
            return;
        };

        if let Err(err) = writer.finalize() {
            cleanup_temp_file(&self.path);
            let message = err.to_string();
            send_transcription_result(
                Err(AudioError::Wav(message)),
                self.rt_handle.clone(),
                self.sender.clone(),
            );
            return;
        }

        transcribe_wav_file_with_handle(
            self.path.clone(),
            self.rt_handle.clone(),
            self.sender.clone(),
        );
    }
}

impl Drop for TranscriptionHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn list_input_devices() -> Result<Vec<InputDeviceInfo>, AudioError> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|err| AudioError::Device(err.to_string()))?;

    let mut infos = Vec::new();
    for device in devices {
        let description = match device.description() {
            Ok(description) => description,
            Err(err) => {
                tracing::warn!(%err, "Failed to read audio device description");
                continue;
            }
        };
        let name = description.name().to_string();
        let id = match device.id() {
            Ok(id) => id.to_string(),
            Err(err) => {
                tracing::warn!(%err, "Failed to read audio device id; falling back to name");
                name.clone()
            }
        };

        infos.push(InputDeviceInfo { id, name });
    }

    Ok(infos)
}

pub fn start_transcription(
    device_id: Option<String>,
    callback: TranscriptionCallback,
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
) -> Result<TranscriptionHandle, AudioError> {
    let host = cpal::default_host();
    let device = match device_id {
        Some(id) => find_input_device_by_id(&host, &id)?,
        None => {
            let device = host
                .default_input_device()
                .ok_or_else(|| AudioError::Device("No default input device available".to_string()))?;
            let name = match device.description() {
                Ok(description) => description.name().to_string(),
                Err(err) => {
                    tracing::warn!(%err, "Failed to read default input device description");
                    "unknown".to_string()
                }
            };
            tracing::info!(%name, "Selected default input device");
            device
        }
    };

    let supported_config = device
        .default_input_config()
        .map_err(|err| AudioError::Config(err.to_string()))?;
    let sample_format = supported_config.sample_format();
    let config: StreamConfig = supported_config.into();

    let wav_spec = hound::WavSpec {
        channels: config.channels,
        sample_rate: config.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let path = build_temp_path();
    let file = File::create(&path).map_err(|err| AudioError::Io(err.to_string()))?;
    let writer =
        hound::WavWriter::new(BufWriter::new(file), wav_spec).map_err(|err| {
            AudioError::Wav(err.to_string())
        })?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    let err_fn = |err| {
        tracing::error!(%err, "Audio stream error");
    };

    let stream = build_input_stream(&device, &config, sample_format, writer.clone(), err_fn)?;
    stream
        .play()
        .map_err(|err| AudioError::Config(err.to_string()))?;

    Ok(TranscriptionHandle {
        stream: Some(stream),
        writer,
        path,
        callback,
        sender,
        rt_handle: rt.handle().clone(),
        stopped: AtomicBool::new(false),
    })
}

fn build_input_stream(
    device: &cpal::Device,
    config: &StreamConfig,
    sample_format: SampleFormat,
    writer: Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, AudioError> {
    let stream = match sample_format {
        SampleFormat::F32 => {
            let writer = writer.clone();
            device.build_input_stream(
                config,
                move |data: &[f32], _| write_input_data_f32(data, &writer),
                err_fn,
                None,
            )
        }
        SampleFormat::I16 => {
            let writer = writer.clone();
            device.build_input_stream(
                config,
                move |data: &[i16], _| write_input_data_i16(data, &writer),
                err_fn,
                None,
            )
        }
        SampleFormat::U16 => {
            let writer = writer.clone();
            device.build_input_stream(
                config,
                move |data: &[u16], _| write_input_data_u16(data, &writer),
                err_fn,
                None,
            )
        }
        _ => {
            return Err(AudioError::Config(format!(
                "Unsupported sample format: {:?}",
                sample_format
            )));
        }
    };

    stream.map_err(|err| AudioError::Config(err.to_string()))
}

fn write_input_data_f32(
    data: &[f32],
    writer: &Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>,
) {
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in data {
                let sample = (sample * i16::MAX as f32)
                    .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                let _ = writer.write_sample(sample);
            }
        }
    }
}

fn write_input_data_i16(
    data: &[i16],
    writer: &Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>,
) {
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in data {
                let _ = writer.write_sample(sample);
            }
        }
    }
}

fn write_input_data_u16(
    data: &[u16],
    writer: &Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>,
) {
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in data {
                let sample = sample as i32 - 32_768;
                let _ = writer.write_sample(sample as i16);
            }
        }
    }
}

fn build_temp_path() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let filename = format!(
        "rift_transcription_{}_{}.wav",
        std::process::id(),
        timestamp
    );
    std::env::temp_dir().join(filename)
}

fn cleanup_temp_file(path: &PathBuf) {
    if let Err(err) = std::fs::remove_file(path) {
        tracing::warn!(%err, path = %path.display(), "Failed to remove transcription temp file");
    }
}

fn find_input_device_by_id(
    host: &cpal::Host,
    device_id: &str,
) -> Result<cpal::Device, AudioError> {
    if let Ok(parsed_id) = device_id.parse::<cpal::DeviceId>() {
        if let Some(device) = host.device_by_id(&parsed_id) {
            return Ok(device);
        }
    }

    let devices = host
        .input_devices()
        .map_err(|err| AudioError::Device(err.to_string()))?;

    for device in devices {
        let description = match device.description() {
            Ok(description) => description,
            Err(err) => {
                tracing::warn!(%err, "Failed to read audio device description");
                continue;
            }
        };
        let name = description.name().to_string();
        let id = device
            .id()
            .map(|id| id.to_string())
            .unwrap_or_else(|_| name.clone());

        if id == device_id || name == device_id {
            return Ok(device);
        }
    }

    Err(AudioError::Device(format!(
        "No input device matches id {device_id}"
    )))
}

pub fn transcribe_wav_file(
    path: PathBuf,
    callback: TranscriptionCallback,
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
) {
    let _ = callback;
    transcribe_wav_file_with_handle(path, rt.handle().clone(), sender);
}

fn transcribe_wav_file_with_handle(
    path: PathBuf,
    rt_handle: tokio::runtime::Handle,
    sender: Sender<AsyncResult>,
) {
    const STT_URL: &str = "http://localhost:8000/stt";
    let sender_for_result = sender.clone();
    let handle_for_result = rt_handle.clone();
    rt_handle.spawn(async move {
        let result = async {
            let data =
                tokio::fs::read(&path).await.map_err(|err| AudioError::Io(err.to_string()))?;

            let part = reqwest::multipart::Part::bytes(data)
                .file_name("audio.wav")
                .mime_str("audio/wav")
                .map_err(|err| AudioError::Io(err.to_string()))?;
            let form = reqwest::multipart::Form::new().part("file", part);

            let client = reqwest::Client::new();
            let response = client
                .post(STT_URL)
                .multipart(form)
                .send()
                .await
                .map_err(|err| AudioError::Network(err.to_string()))?;

            let status = response.status();
            let body = response
                .text()
                .await
                .map_err(|err| AudioError::Network(err.to_string()))?;

            if !status.is_success() {
                return Err(AudioError::Network(format!(
                    "POST {STT_URL} failed with status {}: {body}",
                    status.as_u16()
                )));
            }

            #[derive(serde::Deserialize)]
            struct TranscriptionResponse {
                text: String,
            }

            let response: TranscriptionResponse =
                serde_json::from_str(&body).map_err(|err| {
                    AudioError::Network(format!("Failed to parse response: {err}"))
                })?;

            Ok(response.text)
        }
        .await;

        if let Err(err) = tokio::fs::remove_file(&path).await {
            tracing::warn!(%err, path = %path.display(), "Failed to remove transcription temp file");
        }

        send_transcription_result(result, handle_for_result, sender_for_result);
    });
}

fn send_transcription_result(
    result: Result<String, AudioError>,
    rt_handle: tokio::runtime::Handle,
    sender: Sender<AsyncResult>,
) {
    let async_result = match result {
        Ok(text) => Ok(text),
        Err(AudioError::Network(message)) => Err(AsyncError::Network {
            url: "http://localhost:8000/stt".to_string(),
            method: "POST",
            status: None,
            message,
        }),
        Err(err) => Err(AsyncError::Audio {
            message: err.to_string(),
        }),
    };

    rt_handle.spawn(async move {
        if let Err(err) = sender
            .send(AsyncResult {
                result: async_result,
                callback: transcription_async_callback,
            })
            .await
        {
            tracing::warn!(%err, "Failed to forward transcription result");
        }
    });
}

fn transcription_async_callback(
    result: Result<String, AsyncError>,
    state: &mut EditorState,
) {
    let callback = match state.transcription_handle.as_ref() {
        Some(handle) => handle.callback,
        None => {
            tracing::warn!("Transcription handle missing; dropping result");
            return;
        }
    };
    let audio_result = result.map_err(map_async_error);
    callback(audio_result, state);
    state.transcription_handle = None;
}

fn map_async_error(err: AsyncError) -> AudioError {
    match err {
        AsyncError::Network {
            url,
            method,
            status,
            message,
        } => {
            let status = status
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            AudioError::Network(format!(
                "{method} {url} failed with status {status}: {message}"
            ))
        }
        AsyncError::Process {
            program,
            args,
            status,
            stderr,
            message,
        } => {
            let status = status
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let args = args.join(" ");
            AudioError::Io(format!(
                "{program} {args} failed with status {status}: {message} {stderr}"
            ))
        }
        AsyncError::Audio { message } => AudioError::Io(message),
    }
}
