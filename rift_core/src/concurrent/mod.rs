use tokio::sync::mpsc::{Receiver, Sender};

use crate::state::EditorState;

pub mod web_api;

pub struct AsyncHandle {
    pub sender: Sender<AsyncResult>,
    pub receiver: Receiver<AsyncResult>,
}

pub struct AsyncResult {
    pub result: String,
    pub callback: fn(String, state: &mut EditorState),
}
