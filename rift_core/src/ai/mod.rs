use serde_json::Value;

use crate::{
    actions::{perform_action, Action},
    concurrent::web_api::post_request,
    state::EditorState,
};

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

pub fn generate_qwen_fim_prompt(prefix: &str, suffix: &str) -> String {
    String::new() + "<|fim_prefix|>" + prefix + "<|fim_suffix|>" + suffix + "<|fim_middle|>"
}

pub fn ollama_fim(state: &mut EditorState) {
    let (buffer, instance) = state.get_buffer_by_id(state.buffer_idx.unwrap());
    let content = buffer.get_content("\n".into());
    let byte_idx = buffer.byte_index_from_cursor(&instance.cursor, "\n");
    let (prefix, suffix) = content.split_at(byte_idx);

    let request = OllamaGenerate {
        model: "qwen2.5-coder:1.5b".into(),
        prompt: generate_qwen_fim_prompt(prefix, suffix),
        template: "".into(),
        system: "".into(),
        stream: false,
        raw: true,
        options: serde_json::json!({
            "seed": 42,
            "temperature": 0.7,
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
        "http://localhost:11434/api/generate".into(),
        body,
        |response, state, lsp_handle| {
            let response: Value = serde_json::from_str(&response).unwrap();
            let fim_result = response["response"].as_str().unwrap().to_string();
            tracing::info!("FIM: {}", fim_result);
            perform_action(Action::InsertTextAtCursor(fim_result), state, lsp_handle);
        },
        &state.rt,
        state.async_handle.sender.clone(),
    );
}
