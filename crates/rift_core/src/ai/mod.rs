use std::collections::HashMap;

use serde_json::Value;

use crate::{concurrent::web_api::post_request, state::EditorState};

pub struct AIState {
    pub model_name: String,
    pub url: String,
    pub prompts: HashMap<String, String>,
    pub input: String,
    pub output: String,
    pub seed: usize,
    pub temperature: f32,
}

impl Default for AIState {
    fn default() -> Self {
        Self {
            model_name: "qwen2.5-coder:0.5b".into(),
            url: "http://localhost:11434/api/generate".into(),
            prompts: HashMap::from([(
                "fim".into(),
                "{input}<|fim_prefix|>{prefix}<|fim_suffix|>{suffix}<|fim_middle|>".into(),
            )]),
            input: String::new(),
            output: String::new(),
            seed: 42,
            temperature: 0.7,
        }
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

    let prompt_fmt = state.ai_state.prompts["fim"].clone();
    let prompt = formatter(
        prompt_fmt,
        HashMap::from([
            ("input".into(), state.ai_state.input.clone()),
            ("prefix".into(), prefix.into()),
            ("suffix".into(), suffix.into()),
        ]),
    );

    let request = OllamaGenerate {
        model: state.ai_state.model_name.clone(),
        prompt,
        template: "".into(),
        system: "".into(),
        stream: false,
        raw: true,
        options: serde_json::json!({
            "seed": state.ai_state.seed,
            "temperature": state.ai_state.temperature,
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
            ]
        }),
    };
    let body = serde_json::to_string(&request).unwrap();

    post_request(
        state.ai_state.url.clone(),
        body,
        |response, state, _lsp_handle| {
            let response: Value = serde_json::from_str(&response).unwrap();
            let fim_result = response["response"].as_str().unwrap().to_string();
            state.ai_state.output = fim_result;
        },
        &state.rt,
        state.async_handle.sender.clone(),
    );
}
