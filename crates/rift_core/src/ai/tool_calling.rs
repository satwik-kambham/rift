use std::collections::HashMap;

use chrono::Local;
use tokio::process::Command;
use tokio::sync::mpsc::Sender;

use crate::ai::LLMChatMessage;
use crate::concurrent::AsyncResult;
use crate::{buffer::instance::Language, lsp::client::LSPClientHandle, state::EditorState};

pub async fn run_command(command: &str) -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .await
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr)
}

pub fn get_datetime() -> String {
    let now = Local::now();
    now.format("%d/%m/%Y %H:%M").to_string()
}

pub fn get_tools() -> serde_json::Value {
    serde_json::json!([
        {
            "type": "function",
            "function": {
                "name": "run_command",
                "description": "Run a shell command and return the output",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {"type": "string"}
                    },
                    "required": ["command"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "get_datetime",
                "description": "Get the current date and time",
                "parameters": {
                    "type": "object",
                    "properties": {}
                }
            }
        }
    ])
}

pub async fn get_tool_response(tool_name: &str, tool_arguments: &serde_json::Value) -> String {
    match tool_name {
        "run_command" => run_command(tool_arguments["command"].as_str().unwrap()).await,
        "get_datetime" => get_datetime(),
        _ => "Unknown Tool".to_string(),
    }
}

pub fn tool_requires_approval(tool_name: &str, _tool_arguments: &serde_json::Value) -> bool {
    match tool_name {
        "run_command" => true,
        "get_datetime" => false,
        _ => false,
    }
}

pub fn handle_tool_calls(
    tool_name: String,
    tool_arguments: String,
    state: &mut EditorState,
) {
    let  tool_args = serde_json::from_str(&tool_arguments).unwrap();
    handle_tool_calls_async(
        tool_name.to_string(),
        tool_args,
        |response, state, _lsp_handle| {
            let tool_response: LLMChatMessage = serde_json::from_str(&response).unwrap();
            state.ai_state.chat_state.history.push(tool_response);
            crate::ai::ollama_chat_send(state);
        },
        &state.rt,
        state.async_handle.sender.clone(),
    );
}

pub fn handle_tool_calls_async(
    tool_name: String,
    tool_arguments: serde_json::Value,
    callback: fn(
        String,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
) {
    rt.spawn(async move {
        let tool_response = get_tool_response(&tool_name, &tool_arguments).await;
        let tool_response = LLMChatMessage {
            role: "tool".into(),
            content: tool_response,
            tool_calls: None,
            name: Some(tool_name.to_string()),
        };
        let tool_response = serde_json::to_string(&tool_response).unwrap();

        sender
            .send(AsyncResult {
                result: tool_response,
                callback,
            })
            .await
            .unwrap();
    });
}
