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
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<serde_json::Value>,
    /// Tool name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
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
    pub pending_tool_calls: Vec<(String, String)>,
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
            model_name: "google/gemini-2.5-flash-preview-05-20".into(),
            url: "https://openrouter.ai/api/v1/chat/completions".into(),
            input: String::new(),
            history: vec![],
            seed: 42,
            temperature: 0.7,
        }
    }
}

impl Default for ChatState {
    fn default() -> Self {
        ChatState::ollama()
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
                    let requires_approval =
                        tool_calling::tool_requires_approval(tool_name, &tool_args);
                    if requires_approval {
                        state.ai_state.pending_tool_calls.push((
                            tool_name.to_string(),
                            serde_json::to_string(&tool_args).unwrap(),
                        ));
                    } else {
                        tool_calling::handle_tool_calls_async(
                            tool_name.to_string(),
                            tool_args,
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
    let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());

    let prompt = formatter(
        state.ai_state.chat_state.input.clone(),
        HashMap::from([("source".into(), buffer.get_content("\n".to_string()))]),
    );

    state.ai_state.chat_state.history.push(LLMChatMessage {
        role: "user".into(),
        content: prompt,
        tool_calls: None,
        name: None,
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

    /// Whether to include usage information in the response.
    usage: bool,

    /// Enable streaming of results.
    stream: bool,

    /// Sampling temperature (range: [0, 2]).
    temperature: Option<f32>,

    /// Seed for deterministic outputs.
    seed: Option<usize>,
}

pub fn openrouter_chat(state: &mut EditorState) {
    let auth_token = std::env::var("OPENROUTER_KEY").unwrap();
    let (buffer, _instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());

    let prompt = formatter(
        state.ai_state.chat_state.input.clone(),
        HashMap::from([("source".into(), buffer.get_content("\n".to_string()))]),
    );

    state.ai_state.chat_state.history.push(LLMChatMessage {
        role: "user".into(),
        content: prompt,
        tool_calls: None,
        name: None,
    });
    let request = OpenRouterChat {
        model: state.ai_state.chat_state.model_name.clone(),
        messages: state.ai_state.chat_state.history.clone(),
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
            let response: Value = serde_json::from_str(&response).unwrap();
            let choices = response["choices"].as_array().unwrap();
            let message: LLMChatMessage =
                serde_json::from_value(choices[0]["message"].clone()).unwrap();
            state.ai_state.chat_state.history.push(message);
        },
        &state.rt,
        state.async_handle.sender.clone(),
    );
}
