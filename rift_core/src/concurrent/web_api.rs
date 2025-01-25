use tokio::sync::mpsc::Sender;

use crate::{lsp::client::LSPClientHandle, state::EditorState};

use super::AsyncResult;

pub fn get_request(
    url: String,
    callback: fn(String, state: &mut EditorState, lsp_handle: &mut LSPClientHandle),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
) {
    rt.spawn(async move {
        let response = reqwest::get(url).await.unwrap();
        let content = response.text().await.unwrap();
        sender
            .send(AsyncResult {
                result: content,
                callback,
            })
            .await
            .unwrap();
    });
}

pub fn post_request(
    url: String,
    body: String,
    callback: fn(String, state: &mut EditorState, lsp_handle: &mut LSPClientHandle),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
) {
    rt.spawn(async move {
        let client = reqwest::Client::new();
        let response = client.post(url).body(body).send().await.unwrap();
        let content = response.text().await.unwrap();
        sender
            .send(AsyncResult {
                result: content,
                callback,
            })
            .await
            .unwrap();
    });
}
