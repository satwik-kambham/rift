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
}

/// Starts lsp
pub async fn start_lsp() -> Result<LSPClientHandle> {
    let mut child = Command::new("rust-analyzer")
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
            let mut body = String::default();

            match message_content {
                OutgoingMessage::Request(request) => {
                    let request_body = types::RequestMessage {
                        jsonrpc: "2.0".to_string(),
                        id: next_id(),
                        method: request.method,
                        params: request.params,
                    };
                    body = serde_json::to_string(&request_body).unwrap();
                }
                OutgoingMessage::Notification(notification) => {
                    let notification_body = types::NotificationMessage {
                        jsonrpc: "2.0".to_string(),
                        method: notification.method,
                        params: notification.params,
                    };
                    body = serde_json::to_string(&notification_body).unwrap();
                }
                OutgoingMessage::Response(response) => {
                    let notification_body = types::ResponseMessage {
                        jsonrpc: "2.0".to_string(),
                        id: response.id,
                        result: response.result,
                        error: response.error,
                    };
                    body = serde_json::to_string(&notification_body).unwrap();
                }
            }

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
            }
        }
    });

    // Read incoming errors from the lsp stderr
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        while let Ok(bytes_read) = reader.read_line(&mut line).await {
            if bytes_read > 0 {
                println!("{}", line);
            }
        }
    });

    Ok(LSPClientHandle {
        sender: outgoing_tx,
        reciever: incoming_rx,
        pending_id: 0,
        pending_requests: HashMap::new(),
    })
}

impl LSPClientHandle {
    pub async fn send_request(&self, method: String, params: Option<Value>) -> Result<()> {
        self.sender
            .send(OutgoingMessage::Request(Request { method, params }))
            .await?;
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

    pub async fn send_notification(&self, method: String, params: Option<Value>) -> Result<()> {
        self.sender
            .send(OutgoingMessage::Notification(Notification {
                method,
                params,
            }))
            .await?;
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

    /// Send initialize request and wait for response
    pub async fn init_lsp(&mut self) {
        self.send_request(
            "initialize".to_string(),
            Some(json!({
                "processId": process::id(),
                "rootUri": "file:////home/satwik/Documents/rift",
                "capabilities": {
                    "textDocument": {
                        "completion": {
                            "completionItem": {
                                "snippetSupport": true,
                            }
                        }
                    }
                }
            })),
        )
        .await
        .unwrap();

        loop {
            if let Some(response) = self.recv_message().await {
                if let IncomingMessage::Response(message) = response {
                    println!("{:#?}", message);
                }
                break;
            }
        }
    }
}
