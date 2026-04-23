use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

use crate::error::{CloudflareError, CloudflareResult};

/// Builds the Cloudflare Workers AI base URL for the given account ID.
fn base_url(account_id: &str) -> String {
    format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/ai/v1",
        account_id
    )
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
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

pub struct CloudflareClient {
    http_client: HttpClient,
    api_token: String,
    account_id: String,
}

impl CloudflareClient {
    pub fn new(account_id: impl Into<String>, api_token: impl Into<String>) -> Self {
        Self {
            http_client: HttpClient::new(),
            api_token: api_token.into(),
            account_id: account_id.into(),
        }
    }

    pub async fn create_chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> CloudflareResult<ChatCompletionResponse> {
        let url = format!("{}/chat/completions", base_url(&self.account_id));
        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&self.api_token)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(CloudflareError::ApiError {
                message: format!("HTTP {}: {}", status, text),
            });
        }

        response.json().await.map_err(Into::into)
    }

    pub async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> CloudflareResult<reqwest::Response> {
        let req = ChatCompletionRequest {
            stream: Some(true),
            ..request
        };

        let url = format!("{}/chat/completions", base_url(&self.account_id));
        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&self.api_token)
            .json(&req)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(CloudflareError::ApiError {
                message: format!("HTTP {}: {}", status, text),
            });
        }

        Ok(response)
    }
}
