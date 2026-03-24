use tracing::{error, warn};

use crate::{audio, state::EditorState};

use super::{Action, perform_action};

fn insert_transcription_callback(
    result: Result<String, audio::AudioError>,
    state: &mut EditorState,
) {
    match result {
        Ok(text) => {
            perform_action(Action::InsertTextAtCursor(text), state);
        }
        Err(err) => {
            warn!(%err, "Transcription failed");
        }
    }
}

pub fn tts(state: &mut EditorState, text: String) {
    audio::start_tts(text, state);
}

pub fn tts_buffer(state: &mut EditorState) -> Option<String> {
    let (buffer, _instance) = state.get_buffer_by_id_mut(state.buffer_idx?);
    let content = buffer.get_content("\n".to_string());
    perform_action(Action::Tts(content), state);
    None
}

pub fn start_transcription(
    state: &mut EditorState,
    callback: audio::TranscriptionCallback,
) -> Option<String> {
    if let Some(handle) = state.transcription_handle.as_mut() {
        handle.stop();
        warn!("Transcription already in progress");
        state.audio_recording = false;
        return None;
    }

    match audio::start_transcription(
        None,
        callback,
        &state.rt_handle,
        state.async_handle.sender.clone(),
    ) {
        Ok(handle) => {
            state.transcription_handle = Some(handle);
            state.audio_recording = true;
        }
        Err(err) => {
            state.audio_recording = false;
            error!(%err, "Failed to start transcription");
        }
    }
    None
}

pub fn stop_transcription(state: &mut EditorState) {
    if let Some(handle) = state.transcription_handle.as_mut() {
        handle.stop();
    }
    state.audio_recording = false;
}

pub fn insert_transcription(state: &mut EditorState) -> Option<String> {
    perform_action(
        Action::StartTranscription(insert_transcription_callback),
        state,
    );
    None
}
