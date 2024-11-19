use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::buffer::instance::Selection;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestMessage {
    pub jsonrpc: String,
    pub id: usize,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub jsonrpc: String,
    pub id: usize,
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub error: Option<ResponseError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub params: Option<Value>,
}

#[derive(Debug)]
pub struct CompletionItem {
    pub label: String,
    pub edit: TextEdit,
}

#[derive(Debug)]
pub struct TextEdit {
    pub text: String,
    pub range: Selection,
}
