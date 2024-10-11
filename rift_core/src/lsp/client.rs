use std::{
    io::{BufRead, Read, Write},
    process::{self, Command, Stdio},
    sync::atomic::{AtomicU32, Ordering},
};

use serde_json::json;

use super::types::{self, base::CompletionItemClientCapabilities};

static ID: AtomicU32 = AtomicU32::new(0);

pub fn init() {
    ID.fetch_add(1, Ordering::SeqCst);
    let request_body = types::base::RequestMessage {
        jsonrpc: "2.0".to_owned(),
        id: ID.load(Ordering::SeqCst),
        method: "initialize".to_owned(),
        params: serde_json::to_value(types::base::InitializeParams {
            process_id: process::id(),
            root_path: "/home/satwik/Documents/rift".to_owned(),
            capabilities: types::base::ClientCapabilities {
                text_document: types::base::TextDocumentClientCapabilities {
                    completion: types::base::CompletionClientCapabilities {
                        completion_item: CompletionItemClientCapabilities {
                            snippet_support: true,
                        },
                    },
                    hover: types::base::HoverClientCapabilities {
                        content_format: vec!["plaintext".to_owned(), "markdown".to_owned()],
                    },
                },
            },
        })
        .unwrap(),
    };
    let request_body = json!({
        "jsonrpc": "2.0",
        "id": ID.load(Ordering::SeqCst),
        "method": "initialize",
        "params": {
            "processId": process::id(),
            "rootUri": "file:////home/satwik/Documents/rift",
            "capabilities": {
                "textDocument": {
                    "completion": {
                        "completionItem": {
                            "snippetSupport": true,
                        },
                    },
                },
            },
        },
    });
    let body = serde_json::to_string(&request_body).unwrap();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    let request = format!("{}{}", header, body);
    println!("*********\n\n{}\n\n*******\n\n", request);

    let mut process = Command::new("rust-analyzer")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = process.stdin.as_mut().unwrap();
    let stdout = process.stdout.take().unwrap();

    let mut reader = std::io::BufReader::new(stdout);

    stdin.write_all(&request.into_bytes());

    let mut response_header = String::new();
    reader.read_line(&mut response_header);
    let mut empty_line = String::new();
    reader.read_line(&mut empty_line);

    let content_length = response_header.strip_prefix("Content-Length: ").unwrap();
    let content_length: usize = content_length.trim().parse().unwrap();

    let mut body = vec![0; content_length];
    reader.read_exact(&mut body);
    println!("{}", String::from_utf8_lossy(&body));
}
