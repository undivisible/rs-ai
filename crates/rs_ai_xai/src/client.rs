use reqwest::Client as HttpClient;
use rs_ai_cache::CacheConfig;
use serde::{Deserialize, Serialize};

use crate::error::{XaiError, XaiResult};

const XAI_API_BASE: &str = "https://api.x.ai/v1";

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub index: i32,
    pub message: Message,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StreamChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct StreamChoice {
    pub index: i32,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
}

pub struct XaiClient {
    http_client: HttpClient,
    _api_key: String,
}

impl XaiClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http_client: HttpClient::new(),
            _api_key: api_key.into(),
        }
    }

    pub async fn create_chat_completion(
        &self,
        request: ChatCompletionRequest,
        cache_config: Option<&CacheConfig>,
    ) -> XaiResult<ChatCompletionResponse> {
        let mut req_builder = self
            .http_client
            .post(format!("{}/chat/completions", XAI_API_BASE))
            .bearer_auth(&self._api_key);

        // Add conversation routing header if present
        if let Some(config) = cache_config {
            if let Some(conv_id) = &config.xai_conv_id {
                req_builder = req_builder.header("x-grok-conv-id", conv_id);
            }
        }

        let response = req_builder.json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(XaiError::ApiError {
                message: format!("HTTP {}: {}", status, text),
            });
        }

        response.json().await.map_err(Into::into)
    }

    pub async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
        cache_config: Option<&CacheConfig>,
    ) -> XaiResult<reqwest::Response> {
        let mut req = ChatCompletionRequest {
            stream: Some(true),
            ..request
        };
        req.stream = Some(true);

        let mut req_builder = self
            .http_client
            .post(format!("{}/chat/completions", XAI_API_BASE))
            .bearer_auth(&self._api_key);

        // Add conversation routing header if present
        if let Some(config) = cache_config {
            if let Some(conv_id) = &config.xai_conv_id {
                req_builder = req_builder.header("x-grok-conv-id", conv_id);
            }
        }

        let response = req_builder.json(&req).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(XaiError::ApiError {
                message: format!("HTTP {}: {}", status, text),
            });
        }

        Ok(response)
    }
}
