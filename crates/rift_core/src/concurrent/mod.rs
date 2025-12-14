use std::collections::HashMap;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::{buffer::instance::Language, lsp::client::LSPClientHandle, state::EditorState};

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
}

pub struct AsyncHandle {
    pub sender: Sender<AsyncResult>,
    pub receiver: Receiver<AsyncResult>,
}

pub struct AsyncResult {
    pub result: Result<String, AsyncError>,
    pub callback: fn(
        Result<String, AsyncError>,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ),
}

pub fn simple_callback(
    data: String,
    callback: fn(
        Result<String, AsyncError>,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
) {
    rt.spawn(async move {
        sender
            .send(AsyncResult {
                result: Ok(data),
                callback,
            })
            .await
            .unwrap();
    });
}
