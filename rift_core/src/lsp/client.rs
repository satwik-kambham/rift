use anyhow::Result;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    process::{self, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::Command,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::buffer::instance::Cursor;

use super::types;

static ID: AtomicUsize = AtomicUsize::new(0);

fn next_id() -> usize {
    ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug)]
pub enum IncomingMessage {
    Response(types::ResponseMessage),
    Notification(types::NotificationMessage),
}

pub struct Request {
    pub id: usize,
    pub method: String,
    pub params: Option<Value>,
}
pub struct Notification {
    pub method: String,
    pub params: Option<Value>,
}
pub struct Response {
    pub id: usize,
    pub result: Option<Value>,
    pub error: Option<types::ResponseError>,
}

pub enum OutgoingMessage {
    Request(Request),
    Notification(Notification),
    Response(Response),
}

pub struct LSPClientHandle {
    pub sender: Sender<OutgoingMessage>,
    pub reciever: Receiver<IncomingMessage>,
    pub pending_id: usize,
    pub pending_requests: HashMap<usize, IncomingMessage>,
    pub id_method: HashMap<usize, String>,
}

/// Starts lsp
pub async fn start_lsp() -> Result<LSPClientHandle> {
    let mut command = Command::new("rust-analyzer");

    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<OutgoingMessage>(32);
    let (incoming_tx, incoming_rx) = mpsc::channel::<IncomingMessage>(32);

    // Send pending outgoing messages to lsp stdin
    tokio::spawn(async move {
        let mut writer = BufWriter::new(stdin);
        while let Some(message_content) = outgoing_rx.recv().await {
            let body = match message_content {
                OutgoingMessage::Request(request) => {
                    let request_body = types::RequestMessage {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        method: request.method,
                        params: request.params,
                    };
                    serde_json::to_string(&request_body).unwrap()
                }
                OutgoingMessage::Notification(notification) => {
                    let notification_body = types::NotificationMessage {
                        jsonrpc: "2.0".to_string(),
                        method: notification.method,
                        params: notification.params,
                    };
                    serde_json::to_string(&notification_body).unwrap()
                }
                OutgoingMessage::Response(response) => {
                    let notification_body = types::ResponseMessage {
                        jsonrpc: "2.0".to_string(),
                        id: response.id,
                        result: response.result,
                        error: response.error,
                    };
                    serde_json::to_string(&notification_body).unwrap()
                }
            };

            let header = format!("Content-Length: {}\r\n\r\n", body.len());
            let message = format!("{}{}", header, body);
            writer.write_all(&message.into_bytes()).await.unwrap();
            writer.flush().await.unwrap();
        }
    });

    // Read incoming messages from the lsp stdout
    let itx = incoming_tx.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);
        let mut header = String::new();
        while let Ok(bytes_read) = reader.read_line(&mut header).await {
            if bytes_read > 0 {
                // Read empty line
                reader.read_line(&mut String::new()).await.unwrap();

                // Parse content length from header
                let content_length = header.strip_prefix("Content-Length: ").unwrap();
                let content_length: usize = content_length.trim().parse().unwrap();

                // Parse content body
                let mut body = vec![0; content_length];
                reader.read_exact(&mut body).await.unwrap();
                let body = String::from_utf8_lossy(&body);
                let body: Value = serde_json::from_str(&body).unwrap();

                // If id is present then it is a response
                if let Some(_id) = body.get("id") {
                    let response: types::ResponseMessage = serde_json::from_value(body).unwrap();
                    itx.send(IncomingMessage::Response(response)).await.unwrap();
                } else {
                    let notification: types::NotificationMessage =
                        serde_json::from_value(body).unwrap();
                    itx.send(IncomingMessage::Notification(notification))
                        .await
                        .unwrap();
                }

                header = String::new();
            }
        }
    });

    // Read incoming errors from the lsp stderr
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        while let Ok(bytes_read) = reader.read_line(&mut line).await {
            if bytes_read > 0 {
                // tracing::error!("{}", line);
            }
        }
    });

    Ok(LSPClientHandle {
        sender: outgoing_tx,
        reciever: incoming_rx,
        pending_id: 0,
        pending_requests: HashMap::new(),
        id_method: HashMap::new(),
    })
}

impl LSPClientHandle {
    pub async fn send_request(&mut self, method: String, params: Option<Value>) -> Result<()> {
        let id = next_id();
        self.id_method.insert(id, method.clone());
        self.sender
            .send(OutgoingMessage::Request(Request { method, params, id }))
            .await?;
        Ok(())
    }

    pub fn send_request_sync(&mut self, method: String, params: Option<Value>) -> Result<()> {
        let id = next_id();
        self.id_method.insert(id, method.clone());
        self.sender
            .blocking_send(OutgoingMessage::Request(Request { method, params, id }))?;
        Ok(())
    }

    pub async fn send_response(
        &self,
        id: usize,
        result: Option<Value>,
        error: Option<types::ResponseError>,
    ) -> Result<()> {
        self.sender
            .send(OutgoingMessage::Response(Response { id, result, error }))
            .await?;
        Ok(())
    }

    pub fn send_response_sync(
        &self,
        id: usize,
        result: Option<Value>,
        error: Option<types::ResponseError>,
    ) -> Result<()> {
        self.sender
            .blocking_send(OutgoingMessage::Response(Response { id, result, error }))?;
        Ok(())
    }

    pub async fn send_notification(&self, method: String, params: Option<Value>) -> Result<()> {
        self.sender
            .send(OutgoingMessage::Notification(Notification {
                method,
                params,
            }))
            .await?;
        Ok(())
    }

    pub fn send_notification_sync(&self, method: String, params: Option<Value>) -> Result<()> {
        self.sender
            .blocking_send(OutgoingMessage::Notification(Notification {
                method,
                params,
            }))?;
        Ok(())
    }

    pub async fn recv_message(&mut self) -> Option<IncomingMessage> {
        if let Some(message) = self.reciever.recv().await {
            match &message {
                IncomingMessage::Response(response) => {
                    self.pending_requests.insert(response.id, message);
                }
                IncomingMessage::Notification(_notification) => {
                    return Some(message);
                }
            }
        }

        if self.pending_requests.contains_key(&self.pending_id) {
            let message = self.pending_requests.remove(&self.pending_id);
            self.pending_id += 1;
            return message;
        }
        None
    }

    pub fn recv_message_sync(&mut self) -> Option<IncomingMessage> {
        if let Ok(message) = self.reciever.try_recv() {
            match &message {
                IncomingMessage::Response(response) => {
                    self.pending_requests.insert(response.id, message);
                }
                IncomingMessage::Notification(_notification) => {
                    return Some(message);
                }
            }
        }

        if self.pending_requests.contains_key(&self.pending_id) {
            let message = self.pending_requests.remove(&self.pending_id);
            self.pending_id += 1;
            return message;
        }
        None
    }

    pub fn get_initialization_params(&self, workspace_folder: String) -> Value {
        json!({
            "processId": process::id(),
            "rootUri": format!("file:///{}", workspace_folder),
            "capabilities": {
                "textDocument": {
                    "completion": {
                        "completionItem": {
                            "documentationFormat": ["plaintext"],
                            // "insertReplaceSupport": false,
                        },
                    },
                    "hover": {
                        "contentFormat": ["plaintext"],
                    }
                }
            }
        })
    }

    /// Send initialize request and wait for response
    pub async fn init_lsp(&mut self, workspace_folder: String) {
        self.send_request(
            "initialize".to_string(),
            Some(self.get_initialization_params(workspace_folder)),
        )
        .await
        .unwrap();

        loop {
            if let Some(response) = self.recv_message().await {
                if let IncomingMessage::Response(message) = response {
                    // tracing::info!("{:#?}", message);
                    self.send_notification("initialized".to_string(), None)
                        .await
                        .unwrap();
                    break;
                }
                break;
            }
        }
    }

    /// Send initialize request and wait for response
    pub fn init_lsp_sync(&mut self, workspace_folder: String) {
        self.send_request_sync(
            "initialize".to_string(),
            Some(self.get_initialization_params(workspace_folder)),
        )
        .unwrap();

        loop {
            if let Some(response) = self.recv_message_sync() {
                if let IncomingMessage::Response(message) = response {
                    // tracing::info!("{:#?}", message);
                    self.send_notification_sync("initialized".to_string(), None)
                        .unwrap();
                }
                break;
            }
        }
    }

    /// DidOpenTextDocument Notification
    /// method: 'textDocument/didOpen'
    pub fn did_open_text_document(document_path: String, document_content: String) -> Value {
        json!({
            "textDocument": {
                "uri": format!("file:///{}", document_path),
                "languageId": "rust",
                "version": 1,
                "text": document_content,
            }
        })
    }

    /// DidChangeTextDocument Notification
    /// method: 'textDocument/didChange'
    pub fn did_change_text_document(
        document_path: String,
        document_version: usize,
        // range: Selection,
        text: String,
    ) -> Value {
        // let (start, end) = range.in_order();
        json!({
            "textDocument": {
                "uri": format!("file:///{}", document_path),
                "version": document_version,
            },
            "contentChanges": [{
                // "range": {
                //     "start": {
                //         "line": start.row,
                //         "character": start.column,
                //     },
                //     "end": {
                //         "line": end.row,
                //         "character": end.column,
                //     },
                // },
                "text": text,
            }],
        })
    }

    /// Hover Request
    /// method: 'textDocument/hover'
    pub fn hover_request(document_path: String, cursor: Cursor) -> Value {
        json!({
            "textDocument": {
                "uri": format!("file:///{}", document_path),
            },
            "position": {
                "line": cursor.row,
                "character": cursor.column,
            },
        })
    }

    /// Completion Request
    /// method: 'textDocument/completion'
    pub fn completion_request(document_path: String, cursor: Cursor) -> Value {
        json!({
            "textDocument": {
                "uri": format!("file:///{}", document_path),
            },
            "position": {
                "line": cursor.row,
                "character": cursor.column,
            },
        })
    }

    /// Formatting Request
    /// method: 'textDocument/formatting'
    pub fn formatting_request(document_path: String, tab_size: usize) -> Value {
        json!({
            "textDocument": {
                "uri": format!("file:///{}", document_path),
            },
            "options": {
                "tabSize": tab_size,
                "insertSpaces": true,
                "trimTrailingWhitespace": true,
            },
        })
    }
}
