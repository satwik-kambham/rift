use serde::{Deserialize, Serialize};

pub type LSPAny = serde_json::Value;
pub type LSPArray = Vec<LSPAny>;
pub type LSPObject = serde_json::Map<String, LSPAny>;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestMessage {
    pub jsonrpc: String,
    pub id: u32,
    pub method: String,
    pub params: LSPAny,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeParams {
    pub process_id: u32,
    pub root_path: String,
    pub capabilities: ClientCapabilities,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientCapabilities {
    pub text_document: TextDocumentClientCapabilities,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextDocumentClientCapabilities {
    pub completion: CompletionClientCapabilities,
    pub hover: HoverClientCapabilities,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionClientCapabilities {
    pub completion_item: CompletionItemClientCapabilities,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionItemClientCapabilities {
    pub snippet_support: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HoverClientCapabilities {
    pub content_format: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub jsonrpc: String,
    pub id: usize,
    pub result: LSPAny,
    pub error: ResponseError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub jsonrpc: String,
    pub method: String,
    pub params: LSPAny,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
    pub data: LSPAny,
}
