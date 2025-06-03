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

pub async fn get_tool_response(tool_name: &str, tool_arguments: serde_json::Value) -> String {
    match tool_name {
        "run_command" => run_command(tool_arguments["command"].as_str().unwrap()).await,
        "get_datetime" => get_datetime(),
        _ => "Unknown Tool".to_string(),
    }
}

pub fn handle_tool_calls_async(
    llm_response: String,
    callback: fn(
        String,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
) {
    rt.spawn(async move {
        let response: serde_json::Value = serde_json::from_str(&llm_response).unwrap();
        let message: LLMChatMessage = serde_json::from_value(response["message"].clone()).unwrap();
        let tool_calls = message.tool_calls.unwrap();

        let mut tool_responses = vec![];
        for tool_call in tool_calls.as_array().unwrap() {
            let tool_name = tool_call["function"]["name"].as_str().unwrap();
            let tool_args = tool_call["function"]["arguments"].clone();
            let tool_response = get_tool_response(tool_name, tool_args).await;

            tool_responses.push(LLMChatMessage {
                role: "tool".into(),
                content: tool_response,
                tool_calls: None,
                name: Some(tool_name.to_string()),
            });
        }
        let tool_responses = serde_json::to_string(&tool_responses).unwrap();

        sender
            .send(AsyncResult {
                result: tool_responses,
                callback,
            })
            .await
            .unwrap();
    });
}
