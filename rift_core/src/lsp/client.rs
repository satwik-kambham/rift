use anyhow::Result;
use serde_json::Value;
use std::{
    process::Stdio,
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

pub enum OutgoingMessage {
    Request(Request),
    Notification(Notification),
}

pub struct LSPClientHandle {
    pub sender: Sender<OutgoingMessage>,
    pub reciever: Receiver<IncomingMessage>,
}

/// Starts lsp and sends initialize request
pub async fn init_lsp() -> Result<LSPClientHandle> {
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
    })
}

impl LSPClientHandle {}

//         "params": {
//             "processId": process::id(),
//             "rootUri": "file:////home/satwik/Documents/rift",
//             "capabilities": {
//                 "textDocument": {
//                     "completion": {
//                         "completionItem": {
//                             "snippetSupport": true,
//                         },
//                     },
//                 },
//             },
//         },
//     });

//     let mut response_header = String::new();
//     reader.read_line(&mut response_header)?;
//     let mut empty_line = String::new();
//     reader.read_line(&mut empty_line)?;

//     let content_length = response_header.strip_prefix("Content-Length: ").unwrap();
//     let content_length: usize = content_length.trim().parse().unwrap();

//     let mut body = vec![0; content_length];
//     reader.read_exact(&mut body)?;
//     println!("{}", String::from_utf8_lossy(&body));
