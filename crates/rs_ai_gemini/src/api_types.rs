use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct GenerateContentRequest {
    pub contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<GeminiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<ToolConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct GeminiContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub(crate) enum GeminiPart {
    Text { text: String },
    Thought { thought: bool, text: String },
    InlineData { inline_data: InlineData },
    FunctionCall { function_call: FunctionCall },
    FunctionResponse { function_response: FunctionResponse },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct InlineData {
    pub mime_type: String,
    pub data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct FunctionCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct FunctionResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub response: serde_json::Value,
}

#[derive(Serialize)]
pub(crate) struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_config: Option<ThinkingConfig>,
}

#[derive(Serialize)]
pub(crate) struct ThinkingConfig {
    /// Token budget for thinking (Gemini 2.5+).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budget: Option<u32>,
    /// Thinking level for Gemini 3 Flash: "minimal", "low", "medium", "high".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct GeminiTool {
    pub function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Serialize)]
pub(crate) struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Serialize)]
pub(crate) struct ToolConfig {
    pub function_calling_config: FunctionCallingConfig,
}

#[derive(Serialize)]
pub(crate) struct FunctionCallingConfig {
    pub mode: String,
}

// Response types

#[derive(Deserialize, Debug)]
pub(crate) struct GenerateContentResponse {
    pub candidates: Option<Vec<Candidate>>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: Option<UsageMetadata>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Candidate {
    pub content: Option<GeminiContent>,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: Option<u64>,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: Option<u64>,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: Option<u64>,
}
