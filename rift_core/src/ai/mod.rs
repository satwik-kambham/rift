use serde_json::Value;

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

pub async fn ollama_generate(prefix: &str, suffix: &str) -> anyhow::Result<String> {
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

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:11434/api/generate")
        .body(serde_json::to_string(&request).unwrap())
        .send()
        .await?
        .json::<Value>()
        .await?;

    Ok(response["response"].as_str().unwrap().to_string())
}
