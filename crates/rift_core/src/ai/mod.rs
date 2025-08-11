use std::collections::HashMap;

use serde_json::Value;

use crate::{
    concurrent::web_api::{post_request, post_request_json_body_with_bearer_auth},
    state::EditorState,
};

pub mod tool_calling;

pub struct GenerateState {
    pub model_name: String,
    pub url: String,
    pub prompts: HashMap<String, String>,
    pub input: String,
    pub output: String,
    pub seed: usize,
    pub temperature: f32,
    pub num_ctx: usize,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct LLMChatMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<serde_json::Value>,
    /// Tool name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

pub struct ChatState {
    pub provider: String,
    pub model_name: String,
    pub url: String,
    pub input: String,
    pub history: Vec<LLMChatMessage>,
    pub seed: usize,
    pub temperature: f32,
}

#[derive(Default)]
pub struct AIState {
    pub generate_state: GenerateState,
    pub chat_state: ChatState,
    pub pending_tool_calls: Vec<(String, String, Option<String>)>,
    pub full_user_control: bool,
}

impl Default for GenerateState {
    fn default() -> Self {
        Self {
            model_name: "qwen2.5-coder:0.5b-base".into(),
            url: "http://localhost:11434/api/generate".into(),
            prompts: HashMap::from([
                (
                    "file_fim".into(),
                    "{input}<|fim_prefix|>{prefix}<|fim_suffix|>{suffix}<|fim_middle|>".into(),
                ),
                (
                    "repo_fim".into(),
                    "{input}<|fim_prefix|>{prefix}<|fim_suffix|>{suffix}<|fim_middle|>".into(),
                ),
            ]),
            input: String::new(),
            output: String::new(),
            seed: 42,
            temperature: 0.3,
            num_ctx: 4096,
        }
    }
}

impl ChatState {
    pub fn llamacpp() -> Self {
        Self {
            provider: "llamacpp".into(),
            model_name: "qwen3:30b-a3b".into(),
            url: "http://localhost:8080/v1/chat/completions".into(),
            input: String::new(),
            history: vec![],
            seed: 42,
            temperature: 0.3,
        }
    }

    pub fn ollama() -> Self {
        Self {
            provider: "ollama".into(),
            model_name: "qwen3:30b-a3b".into(),
            url: "http://localhost:11434/api/chat".into(),
            input: String::new(),
            history: vec![],
            seed: 42,
            temperature: 0.3,
        }
    }

    pub fn openrouter() -> Self {
        Self {
            provider: "openrouter".into(),
            model_name: "mistralai/devstral-small".into(),
            url: "https://openrouter.ai/api/v1/chat/completions".into(),
            input: String::new(),
            history: vec![],
            seed: 42,
            temperature: 0.2,
        }
    }
}

impl Default for ChatState {
    fn default() -> Self {
        ChatState::llamacpp()
    }
}

pub fn formatter(format: String, args: HashMap<String, String>) -> String {
    let mut result = format;
    for (key, value) in args {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, &value);
    }
    result
}

pub fn create_system_prompt(template: String, state: &mut EditorState) -> String {
    let project_documentation = if std::path::Path::new(&state.workspace_folder)
        .join("RIFT.md")
        .exists()
    {
        std::fs::read_to_string(std::path::Path::new(&state.workspace_folder).join("RIFT.md"))
            .unwrap_or_else(|_| "".into())
    } else {
        "".into()
    };

    return formatter(
        template,
        HashMap::from([
            ("workspace_dir".into(), state.workspace_folder.clone()),
            ("platform".into(), "Linux (NixOS)".into()),
            (
                "file_tree".into(),
                tool_calling::get_file_tree(&state.workspace_folder),
            ),
            ("get_datetime_tool_name".into(), "get_datetime".into()),
            (
                "run_shell_command_tool_name".into(),
                "run_shell_command".into(),
            ),
            ("find_file_tool_name".into(), "find_file".into()),
            ("search_tool_name".into(), "search_workspace".into()),
            ("read_file_tool_name".into(), "read_file".into()),
            ("write_file_tool_name".into(), "write_file".into()),
            ("replace_tool_name".into(), "replace".into()),
            ("project_documentation".into(), project_documentation),
        ]),
    );
}

pub fn get_system_prompt(template_name: &str, state: &mut EditorState) -> String {
    let system_prompt_template = match template_name {
        "default" => include_str!("prompts/SYSTEM.md"),
        "agentic_coding" => include_str!("prompts/AGENTIC_CODING.md"),
        _ => include_str!("prompts/SYSTEM.md"),
    }
    .to_string();
    create_system_prompt(system_prompt_template, state)
}

#[derive(Debug, serde::Serialize)]
/// Request content for ollama generate api
pub struct OllamaGenerate {
    pub model: String,
    pub prompt: String,
    pub template: String,
    pub system: String,
    pub stream: bool,
    pub raw: bool,
    pub options: Value,
}

pub fn ollama_fim(state: &mut EditorState) {
    let (buffer, instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
    let content = buffer.get_content("\n".into());
    let byte_idx = buffer.byte_index_from_cursor(&instance.cursor, "\n");
    let (prefix, suffix) = content.split_at(byte_idx);

    let prompt_fmt = state.ai_state.generate_state.prompts["file_fim"].clone();
    let prompt = formatter(
        prompt_fmt,
        HashMap::from([
            ("input".into(), state.ai_state.generate_state.input.clone()),
            ("prefix".into(), prefix.into()),
            ("suffix".into(), suffix.into()),
        ]),
    );

    let request = OllamaGenerate {
        model: state.ai_state.generate_state.model_name.clone(),
        prompt,
        template: "".into(),
        system: "".into(),
        stream: false,
        raw: true,
        options: serde_json::json!({
            "seed": state.ai_state.generate_state.seed,
            "temperature": state.ai_state.generate_state.temperature,
            "stop": [
                "<|endoftext|>",
                "<|fim_prefix|>",
                "<|fim_suffix|>",
                "<|fim_middle|>",
                "<|fim_pad|>",
                "<|repo_name|>",
                "<|file_sep|>",
                "<|im_start|>",
                "<|im_end|>",
            ],
            "num_ctx": state.ai_state.generate_state.num_ctx,
        }),
    };
    let body = serde_json::to_string(&request).unwrap();

    post_request(
        state.ai_state.generate_state.url.clone(),
        body,
        |response, state, _lsp_handle| {
            let response: Value = serde_json::from_str(&response).unwrap();
            let fim_result = response["response"].as_str().unwrap().to_string();
            state.ai_state.generate_state.output = fim_result;
        },
        &state.rt,
        state.async_handle.sender.clone(),
    );
}

#[derive(serde::Serialize)]
/// Request content for ollama generate api
pub struct LlamacppChat {
    pub model: String,
    pub messages: Vec<LLMChatMessage>,
    pub tools: serde_json::Value,
    pub stream: bool,
    pub temperature: Option<f32>,
    pub seed: Option<usize>,
}

pub fn llamacpp_chat_send(state: &mut EditorState) {
    let request = LlamacppChat {
        model: state.ai_state.chat_state.model_name.clone(),
        messages: state.ai_state.chat_state.history.clone(),
        tools: tool_calling::get_tools(),
        stream: false,
        temperature: Some(state.ai_state.chat_state.temperature),
        seed: Some(state.ai_state.chat_state.seed),
    };
    let body = serde_json::to_string(&request).unwrap();

    post_request(
        state.ai_state.chat_state.url.clone(),
        body,
        |response, state, _lsp_handle| {
            tracing::info!(response);
            let llm_response: Value = serde_json::from_str(&response).unwrap();
            let choices = llm_response["choices"].as_array().unwrap();
            let message: LLMChatMessage =
                serde_json::from_value(choices[0]["message"].clone()).unwrap();
            state.ai_state.chat_state.history.push(message.clone());

            if message.tool_calls.is_some() {
                let response: serde_json::Value = serde_json::from_str(&response).unwrap();
                let choices = response["choices"].as_array().unwrap();
                let message: LLMChatMessage =
                    serde_json::from_value(choices[0]["message"].clone()).unwrap();
                let tool_calls = message.tool_calls.unwrap();

                for tool_call in tool_calls.as_array().unwrap() {
                    let tool_name = tool_call["function"]["name"].as_str().unwrap();
                    let tool_args = tool_call["function"]["arguments"].as_str().unwrap();
                    let tool_args = serde_json::from_str(tool_args).unwrap();
                    let tool_call_id = tool_call["id"].as_str().unwrap();
                    let requires_approval = tool_calling::tool_requires_approval(
                        tool_name,
                        &tool_args,
                        state.ai_state.full_user_control,
                    );
                    if requires_approval {
                        state.ai_state.pending_tool_calls.push((
                            tool_name.to_string(),
                            serde_json::to_string(&tool_args).unwrap(),
                            Some(tool_call_id.to_string()),
                        ));
                    } else {
                        tool_calling::handle_tool_calls_async(
                            tool_name.to_string(),
                            tool_args,
                            Some(tool_call_id.to_string()),
                            state.workspace_folder.clone(),
                            |response, state, _lsp_handle| {
                                let tool_response: LLMChatMessage =
                                    serde_json::from_str(&response).unwrap();
                                state.ai_state.chat_state.history.push(tool_response);
                                llamacpp_chat_send(state);
                            },
                            &state.rt,
                            state.async_handle.sender.clone(),
                        );
                    }
                }
            }
        },
        &state.rt,
        state.async_handle.sender.clone(),
    );
}

pub fn llamacpp_chat(state: &mut EditorState) {
    let prompt = formatter(state.ai_state.chat_state.input.clone(), HashMap::from([]));

    state.ai_state.chat_state.history.push(LLMChatMessage {
        role: "user".into(),
        content: Some(prompt),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    });

    llamacpp_chat_send(state);
}

#[derive(serde::Serialize)]
/// Request content for ollama generate api
pub struct OllamaChat {
    pub model: String,
    pub messages: Vec<LLMChatMessage>,
    pub tools: serde_json::Value,
    pub stream: bool,
    pub options: Value,
}

pub fn ollama_chat_send(state: &mut EditorState) {
    let request = OllamaChat {
        model: state.ai_state.chat_state.model_name.clone(),
        messages: state.ai_state.chat_state.history.clone(),
        tools: tool_calling::get_tools(),
        stream: false,
        options: serde_json::json!({
            "seed": state.ai_state.chat_state.seed,
            "temperature": state.ai_state.chat_state.temperature,
        }),
    };
    let body = serde_json::to_string(&request).unwrap();

    post_request(
        state.ai_state.chat_state.url.clone(),
        body,
        |response, state, _lsp_handle| {
            tracing::info!(response);
            let llm_response: Value = serde_json::from_str(&response).unwrap();
            let message: LLMChatMessage =
                serde_json::from_value(llm_response["message"].clone()).unwrap();
            state.ai_state.chat_state.history.push(message.clone());

            if message.tool_calls.is_some() {
                let response: serde_json::Value = serde_json::from_str(&response).unwrap();
                let message: LLMChatMessage =
                    serde_json::from_value(response["message"].clone()).unwrap();
                let tool_calls = message.tool_calls.unwrap();

                for tool_call in tool_calls.as_array().unwrap() {
                    let tool_name = tool_call["function"]["name"].as_str().unwrap();
                    let tool_args = tool_call["function"]["arguments"].clone();
                    let requires_approval = tool_calling::tool_requires_approval(
                        tool_name,
                        &tool_args,
                        state.ai_state.full_user_control,
                    );
                    if requires_approval {
                        state.ai_state.pending_tool_calls.push((
                            tool_name.to_string(),
                            serde_json::to_string(&tool_args).unwrap(),
                            None,
                        ));
                    } else {
                        tool_calling::handle_tool_calls_async(
                            tool_name.to_string(),
                            tool_args,
                            None,
                            state.workspace_folder.clone(),
                            |response, state, _lsp_handle| {
                                let tool_response: LLMChatMessage =
                                    serde_json::from_str(&response).unwrap();
                                state.ai_state.chat_state.history.push(tool_response);
                                ollama_chat_send(state);
                            },
                            &state.rt,
                            state.async_handle.sender.clone(),
                        );
                    }
                }
            }
        },
        &state.rt,
        state.async_handle.sender.clone(),
    );
}

pub fn ollama_chat(state: &mut EditorState) {
    let prompt = formatter(state.ai_state.chat_state.input.clone(), HashMap::from([]));

    state.ai_state.chat_state.history.push(LLMChatMessage {
        role: "user".into(),
        content: Some(prompt),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    });

    ollama_chat_send(state);
}

#[derive(serde::Serialize)]
/// Request content for ollama generate api
pub struct OpenRouterChat {
    /// The model ID to use. If unspecified, the user’s default is used.
    model: String,

    /// Allowed role values: system, developer, user, assistant, tool
    messages: Vec<LLMChatMessage>,

    tools: serde_json::Value,

    /// Whether to include usage information in the response.
    usage: bool,

    /// Enable streaming of results.
    stream: bool,

    /// Sampling temperature (range: [0, 2]).
    temperature: Option<f32>,

    /// Seed for deterministic outputs.
    seed: Option<usize>,
}

pub fn openrouter_chat_send(state: &mut EditorState) {
    let auth_token = std::env::var("OPENROUTER_KEY").unwrap();

    let request = OpenRouterChat {
        model: state.ai_state.chat_state.model_name.clone(),
        messages: state.ai_state.chat_state.history.clone(),
        tools: tool_calling::get_tools(),
        stream: false,
        usage: true,
        temperature: Some(state.ai_state.chat_state.temperature),
        seed: Some(state.ai_state.chat_state.seed),
    };

    let body = serde_json::to_value(&request).unwrap();

    post_request_json_body_with_bearer_auth(
        state.ai_state.chat_state.url.clone(),
        body,
        auth_token,
        |response, state, _lsp_handle| {
            let llm_response: Value = serde_json::from_str(&response).unwrap();
            let choices = llm_response["choices"].as_array().unwrap();
            let message: LLMChatMessage =
                serde_json::from_value(choices[0]["message"].clone()).unwrap();
            state.ai_state.chat_state.history.push(message.clone());

            if message.tool_calls.is_some() {
                let response: serde_json::Value = serde_json::from_str(&response).unwrap();
                let choices = response["choices"].as_array().unwrap();
                let message: LLMChatMessage =
                    serde_json::from_value(choices[0]["message"].clone()).unwrap();
                let tool_calls = message.tool_calls.unwrap();
                for tool_call in tool_calls.as_array().unwrap() {
                    let tool_name = tool_call["function"]["name"].as_str().unwrap();
                    let tool_args = tool_call["function"]["arguments"].as_str().unwrap();
                    let tool_args = serde_json::from_str(tool_args).unwrap();
                    let tool_call_id = tool_call["id"].as_str().unwrap();

                    let requires_approval = tool_calling::tool_requires_approval(
                        tool_name,
                        &tool_args,
                        state.ai_state.full_user_control,
                    );
                    if requires_approval {
                        state.ai_state.pending_tool_calls.push((
                            tool_name.to_string(),
                            serde_json::to_string(&tool_args).unwrap(),
                            Some(tool_call_id.to_string()),
                        ));
                    } else {
                        tool_calling::handle_tool_calls_async(
                            tool_name.to_string(),
                            tool_args,
                            Some(tool_call_id.to_string()),
                            state.workspace_folder.clone(),
                            |response, state, _lsp_handle| {
                                let tool_response: LLMChatMessage =
                                    serde_json::from_str(&response).unwrap();
                                state.ai_state.chat_state.history.push(tool_response);
                                openrouter_chat_send(state);
                            },
                            &state.rt,
                            state.async_handle.sender.clone(),
                        );
                    }
                }
            }
        },
        &state.rt,
        state.async_handle.sender.clone(),
    );
}

pub fn openrouter_chat(state: &mut EditorState) {
    let prompt = formatter(state.ai_state.chat_state.input.clone(), HashMap::from([]));

    state.ai_state.chat_state.history.push(LLMChatMessage {
        role: "user".into(),
        content: Some(prompt),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    });

    openrouter_chat_send(state);
}
