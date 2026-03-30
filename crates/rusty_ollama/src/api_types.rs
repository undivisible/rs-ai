use serde::{Deserialize, Serialize};

// ── Chat request ──

#[derive(Debug, Serialize)]
pub(crate) struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<OllamaMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OllamaTool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OllamaMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OllamaToolCall>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
}

// ── Tools ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OllamaTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: OllamaFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OllamaFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OllamaToolCall {
    pub function: OllamaFunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OllamaFunctionCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

// ── Chat response ──

#[derive(Debug, Deserialize)]
pub(crate) struct OllamaChatResponse {
    #[allow(dead_code)]
    pub model: String,
    pub message: OllamaMessage,
    pub done: bool,
    #[serde(default)]
    pub done_reason: Option<String>,
    #[serde(default)]
    pub eval_count: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u64>,
}

// ── Embedding ──

#[derive(Debug, Serialize)]
pub(crate) struct OllamaEmbedRequest {
    pub model: String,
    pub input: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OllamaEmbedResponse {
    pub embeddings: Vec<Vec<f32>>,
}

// ── List models ──

#[derive(Debug, Deserialize)]
pub(crate) struct OllamaListResponse {
    pub models: Vec<OllamaModelEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OllamaModelEntry {
    pub name: String,
}
