use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    process::{self, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::Command,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::buffer::instance::{Cursor, Selection};

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
    pub receiver: Receiver<IncomingMessage>,
    pub pending_id: usize,
    pub pending_requests: HashMap<usize, IncomingMessage>,
    pub id_method: HashMap<usize, String>,
    pub initialize_capabilities: Value,
}

/// Starts lsp
pub async fn start_lsp(program: &str, args: &[&str]) -> Result<LSPClientHandle> {
    let mut command = Command::new(program);

    let command_display = if args.is_empty() {
        program.to_string()
    } else {
        format!("{} {}", program, args.join(" "))
    };

    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn LSP command `{}`", command_display))?;

    let stdin = child
        .stdin
        .take()
        .context("Failed to open stdin for LSP process")?;
    let stdout = child
        .stdout
        .take()
        .context("Failed to open stdout for LSP process")?;
    let stderr = child
        .stderr
        .take()
        .context("Failed to open stderr for LSP process")?;

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
                // Read content type if present
                let mut content_type = String::new();
                reader.read_line(&mut content_type).await.unwrap();

                // Read empty line if content type is present
                if content_type.starts_with("Content-Type") {
                    reader.read_line(&mut String::new()).await.unwrap();
                }

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
                tracing::error!("{}", line);
            }
        }
    });

    Ok(LSPClientHandle {
        sender: outgoing_tx,
        receiver: incoming_rx,
        pending_id: 0,
        pending_requests: HashMap::new(),
        id_method: HashMap::new(),
        initialize_capabilities: Value::Null,
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
        if let Some(message) = self.receiver.recv().await {
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
        if let Ok(message) = self.receiver.try_recv() {
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
            "rootUri": format!("file://{}", workspace_folder),
            "capabilities": {
                "textDocument": {
                    "synchronization": {
                        "didSave": true,
                    },
                    "completion": {
                        "completionItem": {
                            "snippetSupport": false,
                            "documentationFormat": ["plaintext"],
                            // "insertReplaceSupport": true,
                        },
                    },
                    "hover": {
                        "contentFormat": ["plaintext"],
                    },
                    "signatureHelp": {
                        "signatureInformation": {
                            "documentationFormat": ["plaintext"],
                        },
                    },
                    "publishDiagnostics": {
                        "versionSupport": true,
                        "dataSupport": true,
                    },
                }
            }
        })
    }

    /// Send initialize request and wait for response
    pub fn init_lsp_sync(&mut self, workspace_folder: String) -> Result<()> {
        let timeout_duration = Duration::from_secs(5);
        let timeout_start = Instant::now();

        self.send_request_sync(
            "initialize".to_string(),
            Some(self.get_initialization_params(workspace_folder)),
        )
        .unwrap();

        loop {
            if let Some(response) = self.recv_message_sync() {
                if let IncomingMessage::Response(message) = response {
                    // tracing::info!("{:#?}", message);
                    self.initialize_capabilities = message.result.unwrap()["capabilities"].clone();
                    self.send_notification_sync("initialized".to_string(), Some(json!({})))
                        .unwrap();
                }
                break;
            }

            if timeout_start.elapsed() >= timeout_duration {
                tracing::warn!("LSP Initialization timed out. Disabling LSP");
                bail!("LSP Initialization timed out")
            }
        }

        Ok(())
    }

    /// DidOpenTextDocument Notification
    /// method: 'textDocument/didOpen'
    pub fn did_open_text_document(
        document_path: String,
        language_id: String,
        document_content: String,
    ) -> Value {
        json!({
            "textDocument": {
                "uri": format!("file://{}", document_path),
                "languageId": language_id,
                "version": 1,
                "text": document_content,
            }
        })
    }

    /// DidSaveTextDocument Notification
    /// method: 'textDocument/didSave'
    pub fn did_save_text_document(document_path: String) -> Value {
        json!({
            "textDocument": {
                "uri": format!("file://{}", document_path),
            }
        })
    }

    /// DidChangeTextDocument Notification
    /// method: 'textDocument/didChange'
    pub fn did_change_text_document(
        document_path: String,
        document_version: usize,
        range: Option<Selection>,
        text: String,
    ) -> Value {
        let content_changes = if let Some(range) = range {
            let (start, end) = range.in_order();

            json!({
                "range": {
                    "start": {
                        "line": start.row,
                        "character": start.column,
                    },
                    "end": {
                        "line": end.row,
                        "character": end.column,
                    },
                },
                "text": text,
            })
        } else {
            json!({
                "text": text,
            })
        };

        json!({
            "textDocument": {
                "uri": format!("file://{}", document_path),
                "version": document_version,
            },
            "contentChanges": [content_changes],
        })
    }

    /// Hover Request
    /// method: 'textDocument/hover'
    pub fn hover_request(document_path: String, cursor: Cursor) -> Value {
        json!({
            "textDocument": {
                "uri": format!("file://{}", document_path),
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
                "uri": format!("file://{}", document_path),
            },
            "position": {
                "line": cursor.row,
                "character": cursor.column,
            },
        })
    }

    /// Completion Request
    /// method: 'textDocument/signatureHelp'
    pub fn signature_help_request(document_path: String, cursor: Cursor) -> Value {
        json!({
            "textDocument": {
                "uri": format!("file://{}", document_path),
            },
            "position": {
                "line": cursor.row,
                "character": cursor.column,
            },
        })
    }

    /// Completion Request
    /// method: 'textDocument/definition'
    pub fn go_to_definition_request(document_path: String, cursor: Cursor) -> Value {
        json!({
            "textDocument": {
                "uri": format!("file://{}", document_path),
            },
            "position": {
                "line": cursor.row,
                "character": cursor.column,
            },
        })
    }

    /// Completion Request
    /// method: 'textDocument/references'
    pub fn go_to_references_request(document_path: String, cursor: Cursor) -> Value {
        json!({
            "textDocument": {
                "uri": format!("file://{}", document_path),
            },
            "position": {
                "line": cursor.row,
                "character": cursor.column,
            },
            "context": {
                "includeDeclaration": true,
            },
        })
    }

    /// Formatting Request
    /// method: 'textDocument/formatting'
    pub fn formatting_request(document_path: String, tab_size: usize) -> Value {
        json!({
            "textDocument": {
                "uri": format!("file://{}", document_path),
            },
            "options": {
                "tabSize": tab_size,
                "insertSpaces": true,
                "trimTrailingWhitespace": true,
            },
        })
    }
}
