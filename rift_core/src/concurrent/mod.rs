use std::collections::HashMap;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::{buffer::instance::Language, lsp::client::LSPClientHandle, state::EditorState};

pub mod cli;
pub mod web_api;

pub struct AsyncHandle {
    pub sender: Sender<AsyncResult>,
    pub receiver: Receiver<AsyncResult>,
}

pub struct AsyncResult {
    pub result: String,
    pub callback:
        fn(String, state: &mut EditorState, lsp_handles: &mut HashMap<Language, LSPClientHandle>),
}

pub fn simple_callback(
    data: String,
    callback: fn(
        String,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
) {
    rt.spawn(async move {
        sender
            .send(AsyncResult {
                result: data,
                callback,
            })
            .await
            .unwrap();
    });
}
