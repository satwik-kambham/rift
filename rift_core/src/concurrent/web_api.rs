use tokio::sync::mpsc::Sender;

use crate::state::EditorState;

use super::AsyncResult;

pub fn get_request(
    url: String,
    callback: fn(String, state: &mut EditorState),
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
