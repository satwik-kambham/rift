use std::{collections::HashMap, path::Path};

use chrono::Local;
use similar::{ChangeTag, TextDiff};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;

use crate::ai::LLMChatMessage;
use crate::concurrent::AsyncResult;
use crate::{buffer::instance::Language, lsp::client::LSPClientHandle, state::EditorState};

pub fn is_absolute_path(path: &str) -> bool {
    Path::new(path).is_absolute()
}

pub fn is_in_workspace(workspace_dir: &str, path: &str) -> bool {
    let workspace_path = Path::new(workspace_dir);
    let file_path = Path::new(path);
    file_path.starts_with(workspace_path)
}

pub async fn run_shell_command(workspace_dir: &str, command: &str) -> String {
    match Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(workspace_dir)
        .output()
        .await
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr)
        }
        Err(e) => format!("Error executing command: {}", e),
    }
}

pub fn get_file_tree(workspace_dir: &str) -> String {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg("fd --type f --strip-cwd-prefix --full-path")
        .current_dir(workspace_dir)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    stdout
}

pub fn find_file(workspace_dir: &str, pattern: &str) -> String {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!(
            "fd --type f --strip-cwd-prefix --full-path --absolute-path {}",
            pattern
        ))
        .current_dir(workspace_dir)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout.trim().is_empty() {
        format!("No files found matching pattern: {}\nMake a meaningful change to the pattern before trying again. If still unsucessful try a different approach or request input from the user.", pattern)
    } else {
        stdout
    }
}

pub fn search_workspace(workspace_dir: &str, pattern: &str) -> String {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("rg {}", pattern))
        .current_dir(workspace_dir)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout.trim().is_empty() {
        format!("No matches found for pattern: {}\nMake a meaningful change to the pattern before trying again. If still unsucessful try a different approach or request input from the user.
", pattern)
    } else {
        stdout
    }
}

pub fn read_file(workspace_dir: &str, path: &str) -> String {
    if !is_absolute_path(path) {
        return "Error: path is not absolute".to_string();
    }
    if !is_in_workspace(workspace_dir, path) {
        return "Error: path is not in workspace".to_string();
    }

    match std::fs::read_to_string(path) {
        Ok(buf) => {
            let lines: Vec<String> = buf
                .lines()
                .enumerate()
                .map(|(line_number, line)| format!("{}\t{}", line_number + 1, line))
                .collect();
            format!("{}\n\n{}", path, lines.join("\n"))
        }
        Err(e) => format!("Error reading file '{}': {}", path, e),
    }
}

pub fn write_file(workspace_dir: &str, file_path: &str, content: &str) -> String {
    if !is_absolute_path(file_path) {
        return "Error: file_path is not absolute".to_string();
    }
    if !is_in_workspace(workspace_dir, file_path) {
        return "Error: file_path is not in workspace".to_string();
    }

    let parent_dir = Path::new(file_path).parent().unwrap();
    if let Err(e) = std::fs::create_dir_all(parent_dir) {
        return format!(
            "Error creating parent directories for '{}': {}",
            file_path, e
        );
    }

    match std::fs::write(file_path, content) {
        Ok(_) => format!("Successfully wrote to file: {}", file_path),
        Err(e) => format!("Error writing to file '{}': {}", file_path, e),
    }
}

pub fn replace(
    workspace_dir: &str,
    file_path: &str,
    old_string: &str,
    new_string: &str,
    expected_replacements: Option<usize>,
) -> String {
    if !is_absolute_path(file_path) {
        return "Error: file_path is not absolute.".to_string();
    }
    if !is_in_workspace(workspace_dir, file_path) {
        return "Error: file_path is not in workspace.".to_string();
    }

    let file_content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => return format!("Error reading file '{}': {}", file_path, e),
    };

    let original_content = file_content.clone();
    let replaced_content = original_content.replace(old_string, new_string);

    let actual_replacements = original_content.matches(old_string).count();

    if let Some(expected) = expected_replacements {
        if actual_replacements != expected {
            return format!(
                "Error: Expected {} replacements, but found {} occurrences of the old string.",
                expected, actual_replacements
            );
        }
    } else if actual_replacements == 0 {
        return format!(
            "Error: No occurrences of the old string found in '{}'.",
            file_path
        );
    }

    let file_content = replaced_content;

    match std::fs::write(file_path, file_content) {
        Ok(_) => format!(
            "Successfully replaced content in '{}'. {} occurrences replaced.",
            file_path, actual_replacements
        ),
        Err(e) => format!("Error writing to file '{}': {}", file_path, e),
    }
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
                "name": "run_shell_command",
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
                "name": "find_file",
                "description": "Returns paths matching regex pattern by searching recursively in the workspace folder, returning absolute paths. Does not accept glob patterns.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "pattern": {"type": "string"}
                    },
                    "required": ["pattern"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "search_workspace",
                "description": "Grep search in the current workspace for matching patterns",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "pattern": {"type": "string"}
                    },
                    "required": ["pattern"]
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
        },
        {
            "type": "function",
            "function": {
                "name": "read_file",
                "description": "Get the content of a file with line numbers",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "write_file",
                "description": "Writes content to a specified file replacing the existing content.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"},
                        "content": {"type": "string"},
                    },
                    "required": ["file_path", "content"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "replace",
                "description": "Replaces text within a file. Can replace single or multiple occurrences.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"},
                        "old_string": {"type": "string"},
                        "new_string": {"type": "string"},
                        "expected_replacements": {"type": "number", "optional": true},
                    },
                    "required": ["file_path", "old_string", "new_string"]
                }
            }
        }
    ])
}

pub async fn get_tool_response(
    tool_name: &str,
    tool_arguments: &serde_json::Value,
    workspace_dir: &str,
) -> String {
    match tool_name {
        "run_shell_command" => {
            run_shell_command(workspace_dir, tool_arguments["command"].as_str().unwrap()).await
        }
        "find_file" => find_file(workspace_dir, tool_arguments["pattern"].as_str().unwrap()),
        "search_workspace" => {
            search_workspace(workspace_dir, tool_arguments["pattern"].as_str().unwrap())
        }
        "read_file" => read_file(workspace_dir, tool_arguments["path"].as_str().unwrap()),
        "write_file" => write_file(
            workspace_dir,
            tool_arguments["file_path"].as_str().unwrap(),
            tool_arguments["content"].as_str().unwrap(),
        ),
        "replace" => replace(
            workspace_dir,
            tool_arguments["file_path"].as_str().unwrap(),
            tool_arguments["old_string"].as_str().unwrap(),
            tool_arguments["new_string"].as_str().unwrap(),
            tool_arguments["expected_replacements"]
                .as_u64()
                .map(|u| u as usize),
        ),
        "get_datetime" => get_datetime(),
        _ => "Unknown Tool".to_string(),
    }
}

pub fn get_replace_diff(
    workspace_dir: &str,
    file_path: &str,
    old_string: &str,
    new_string: &str,
    expected_replacements: Option<usize>,
) -> String {
    if !is_absolute_path(file_path) {
        return "Error: file_path is not absolute.".to_string();
    }
    if !is_in_workspace(workspace_dir, file_path) {
        return "Error: file_path is not in workspace.".to_string();
    }

    let file_content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => return format!("Error reading file '{}': {}", file_path, e),
    };

    let original_content = file_content.clone();
    let replaced_content = original_content.replace(old_string, new_string);

    let actual_replacements = original_content.matches(old_string).count();

    if let Some(expected) = expected_replacements {
        if actual_replacements != expected {
            return format!(
                "Error: Expected {} replacements, but found {} occurrences of the old string.",
                expected, actual_replacements
            );
        }
    } else if actual_replacements == 0 {
        return format!(
            "Error: No occurrences of the old string found in '{}'.",
            file_path
        );
    }

    let diff = TextDiff::from_lines(&file_content, &replaced_content);
    let mut diff_output = String::new();
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Equal => " ",
            ChangeTag::Insert => "+",
            ChangeTag::Delete => "-",
        };
        if change.tag() != ChangeTag::Equal {
            diff_output.push_str(&format!(
                "{} {} {}{}",
                change.old_index().unwrap_or_default(),
                change.new_index().unwrap_or_default(),
                sign,
                change
            ));
        }
    }
    diff_output
}

pub fn get_tool_call_preview(
    tool_name: &str,
    tool_arguments: &serde_json::Value,
    workspace_dir: &str,
) -> String {
    match tool_name {
        "run_shell_command" => tool_arguments["command"].as_str().unwrap().to_string(),
        "find_file" => tool_arguments["pattern"].as_str().unwrap().to_string(),
        "search_workspace" => tool_arguments["pattern"].as_str().unwrap().to_string(),
        "read_file" => tool_arguments["path"].as_str().unwrap().to_string(),
        "write_file" => format!(
            "{} {}",
            tool_arguments["file_path"].as_str().unwrap(),
            tool_arguments["content"].as_str().unwrap(),
        ),
        "replace" => get_replace_diff(
            workspace_dir,
            tool_arguments["file_path"].as_str().unwrap(),
            tool_arguments["old_string"].as_str().unwrap(),
            tool_arguments["new_string"].as_str().unwrap(),
            tool_arguments["expected_replacements"]
                .as_u64()
                .map(|u| u as usize),
        ),
        "get_datetime" => get_datetime(),
        _ => "Unknown Tool".to_string(),
    }
}

pub fn tool_requires_approval(
    tool_name: &str,
    _tool_arguments: &serde_json::Value,
    full_user_control: bool,
) -> bool {
    if !full_user_control {
        return match tool_name {
            "run_shell_command" => true,
            "find_file" => false,
            "search_workspace" => false,
            "read_file" => false,
            "write_file" => true,
            "replace" => true,
            "get_datetime" => false,
            _ => true,
        };
    }
    true
}

pub fn handle_tool_calls(
    tool_name: String,
    tool_arguments: String,
    tool_call_id: Option<String>,
    state: &mut EditorState,
    approved: bool,
) {
    if approved {
        let tool_args = serde_json::from_str(&tool_arguments).unwrap();
        handle_tool_calls_async(
            tool_name.to_string(),
            tool_args,
            tool_call_id,
            state.workspace_folder.clone(),
            |response, state, _lsp_handle| {
                let tool_response: LLMChatMessage = serde_json::from_str(&response).unwrap();
                state.ai_state.chat_state.history.push(tool_response);
                if state.ai_state.chat_state.provider == "llamacpp" {
                    crate::ai::llamacpp_chat_send(state);
                } else if state.ai_state.chat_state.provider == "ollama" {
                    crate::ai::ollama_chat_send(state);
                } else if state.ai_state.chat_state.provider == "openrouter" {
                    crate::ai::openrouter_chat_send(state);
                }
            },
            &state.rt,
            state.async_handle.sender.clone(),
        );
    } else {
        let denial_message = LLMChatMessage {
            role: "tool".into(),
            content: Some(format!(
                "Tool call '{}' was denied by user.\n{}",
                tool_name, state.ai_state.chat_state.input
            )),
            tool_calls: None,
            name: Some(tool_name.to_string()),
            tool_call_id,
        };
        state.ai_state.chat_state.input.clear();
        state.ai_state.chat_state.history.push(denial_message);
        if state.ai_state.chat_state.provider == "llamacpp" {
            crate::ai::llamacpp_chat_send(state);
        } else if state.ai_state.chat_state.provider == "ollama" {
            crate::ai::ollama_chat_send(state);
        } else if state.ai_state.chat_state.provider == "openrouter" {
            crate::ai::openrouter_chat_send(state);
        }
    }
}

pub fn handle_tool_calls_async(
    tool_name: String,
    tool_arguments: serde_json::Value,
    tool_call_id: Option<String>,
    workspace_dir: String,
    callback: fn(
        String,
        state: &mut EditorState,
        lsp_handles: &mut HashMap<Language, LSPClientHandle>,
    ),
    rt: &tokio::runtime::Runtime,
    sender: Sender<AsyncResult>,
) {
    rt.spawn(async move {
        let tool_response = get_tool_response(&tool_name, &tool_arguments, &workspace_dir).await;
        let tool_response = LLMChatMessage {
            role: "tool".into(),
            content: Some(tool_response),
            tool_calls: None,
            name: Some(tool_name.to_string()),
            tool_call_id,
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
