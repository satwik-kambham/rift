use tokio::sync::mpsc::{Receiver, Sender};

use crate::state::EditorState;

pub mod cli;
pub mod web_api;

#[derive(Debug)]
pub enum AsyncError {
    Network {
        url: String,
        method: &'static str,
        status: Option<u16>,
        message: String,
    },
    Process {
        program: String,
        args: Vec<String>,
        status: Option<i32>,
        stderr: String,
        message: String,
    },
    Audio {
        message: String,
    },
}

#[derive(Debug)]
pub enum AsyncPayload {
    Text(String),
    Bytes(Vec<u8>),
}

pub struct AsyncHandle {
    pub sender: Sender<AsyncResult>,
    pub receiver: Receiver<AsyncResult>,
}

pub struct AsyncResult {
    pub result: Result<AsyncPayload, AsyncError>,
    pub callback: fn(Result<AsyncPayload, AsyncError>, state: &mut EditorState),
}

pub fn simple_callback(
    data: String,
    callback: fn(Result<AsyncPayload, AsyncError>, state: &mut EditorState),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
) {
    rt.spawn(async move {
        sender
            .send(AsyncResult {
                result: Ok(AsyncPayload::Text(data)),
                callback,
            })
            .await
            .unwrap();
    });
}
