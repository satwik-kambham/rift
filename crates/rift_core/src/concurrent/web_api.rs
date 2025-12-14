use std::collections::HashMap;

use tokio::sync::mpsc::Sender;

use crate::{buffer::instance::Language, lsp::client::LSPClientHandle, state::EditorState};

use super::{AsyncError, AsyncResult};

pub fn get_request(
    url: String,
    callback: fn(
        Result<String, AsyncError>,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
) {
    rt.spawn(async move {
        let url_for_err = url.clone();
        let result = async {
            let response = reqwest::get(&url).await.map_err(|err| AsyncError::Network {
                url: url_for_err.clone(),
                method: "GET",
                status: None,
                message: err.to_string(),
            })?;
            let status = response.status();
            let content = response.text().await.map_err(|err| AsyncError::Network {
                url: url_for_err.clone(),
                method: "GET",
                status: Some(status.as_u16()),
                message: err.to_string(),
            })?;

            if !status.is_success() {
                return Err(AsyncError::Network {
                    url: url_for_err,
                    method: "GET",
                    status: Some(status.as_u16()),
                    message: content,
                });
            }

            Ok(content)
        }
        .await;

        sender
            .send(AsyncResult {
                result,
                callback,
            })
            .await
            .unwrap();
    });
}

pub fn post_request(
    url: String,
    body: String,
    callback: fn(
        Result<String, AsyncError>,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
) {
    rt.spawn(async move {
        let client = reqwest::Client::new();
        let url_for_err = url.clone();
        let result = async {
            let response = client
                .post(&url)
                .body(body)
                .send()
                .await
                .map_err(|err| AsyncError::Network {
                    url: url_for_err.clone(),
                    method: "POST",
                    status: None,
                    message: err.to_string(),
                })?;
            let status = response.status();
            let content = response.text().await.map_err(|err| AsyncError::Network {
                url: url_for_err.clone(),
                method: "POST",
                status: Some(status.as_u16()),
                message: err.to_string(),
            })?;

            if !status.is_success() {
                return Err(AsyncError::Network {
                    url: url_for_err,
                    method: "POST",
                    status: Some(status.as_u16()),
                    message: content,
                });
            }

            Ok(content)
        }
        .await;

        sender
            .send(AsyncResult {
                result,
                callback,
            })
            .await
            .unwrap();
    });
}

pub fn post_request_json_body_with_bearer_auth(
    url: String,
    body: serde_json::Value,
    bearer_auth_token: String,
    callback: fn(
        Result<String, AsyncError>,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
) {
    rt.spawn(async move {
        let client = reqwest::Client::new();
        let url_for_err = url.clone();
        let result = async {
            let response = client
                .post(&url)
                .bearer_auth(bearer_auth_token)
                .json(&body)
                .send()
                .await
                .map_err(|err| AsyncError::Network {
                    url: url_for_err.clone(),
                    method: "POST",
                    status: None,
                    message: err.to_string(),
                })?;
            let status = response.status();
            let content = response.text().await.map_err(|err| AsyncError::Network {
                url: url_for_err.clone(),
                method: "POST",
                status: Some(status.as_u16()),
                message: err.to_string(),
            })?;

            if !status.is_success() {
                return Err(AsyncError::Network {
                    url: url_for_err,
                    method: "POST",
                    status: Some(status.as_u16()),
                    message: content,
                });
            }

            Ok(content)
        }
        .await;

        sender
            .send(AsyncResult {
                result,
                callback,
            })
            .await
            .unwrap();
    });
}
