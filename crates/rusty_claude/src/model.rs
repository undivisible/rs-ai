use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};

use rusty_ai::capability::{Capability, CapabilitySet};
use rusty_ai::error::{AiError, AiResult};
use rusty_ai::model::{GenerateOptions, LanguageModel};
use rusty_ai::prompt::Prompt;
use rusty_ai::stream::AiStream;
use rusty_ai::structured::GenerateResult;

use crate::convert;
use crate::stream_parser;

const ANTHROPIC_VERSION: &str = "2023-06-01";
const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";

/// A Claude language model served by Anthropic.
pub struct ClaudeModel {
    api_key: SecretString,
    model_id: String,
    capabilities: CapabilitySet,
    client: reqwest::Client,
    base_url: String,
}

impl ClaudeModel {
    /// Create a new `ClaudeModel`.
    ///
    /// `api_key` is the Anthropic API key and `model_id` is a model identifier
    /// such as `"claude-sonnet-4-20250514"`.
    pub fn new(api_key: impl Into<String>, model_id: &str) -> Self {
        let capabilities = CapabilitySet::new()
            .with(Capability::TextInput)
            .with(Capability::TextOutput)
            .with(Capability::ImageInput)
            .with(Capability::Streaming)
            .with(Capability::ToolCalling);

        Self {
            api_key: SecretString::from(api_key.into()),
            model_id: model_id.to_string(),
            capabilities,
            client: reqwest::Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Override the base URL (useful for proxies or testing).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Send a request to the Anthropic Messages API.
    async fn send_request(
        &self,
        prompt: Prompt,
        options: &GenerateOptions,
        stream: bool,
    ) -> AiResult<reqwest::Response> {
        let request = convert::build_request(&self.model_id, prompt, options, stream);

        let body =
            serde_json::to_string(&request).map_err(|e| AiError::Serialization(e.to_string()))?;

        tracing::debug!(model = %self.model_id, stream = stream, "Sending request to Anthropic");

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", self.api_key.expose_secret())
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| AiError::Transport {
                message: e.to_string(),
                source: Some(Box::new(e)),
            })?;

        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body_text = response.text().await.unwrap_or_default();

            // Try to parse structured error from Anthropic.
            let message =
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body_text) {
                    parsed["error"]["message"]
                        .as_str()
                        .unwrap_or(&body_text)
                        .to_string()
                } else {
                    body_text
                };

            if status_code == 401 {
                return Err(AiError::AuthError { message });
            }
            if status_code == 429 {
                return Err(AiError::RateLimit { retry_after: None });
            }

            return Err(AiError::ProviderError {
                provider: "anthropic".to_string(),
                status: Some(status_code),
                message,
            });
        }

        Ok(response)
    }
}

#[async_trait]
impl LanguageModel for ClaudeModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "anthropic"
    }

    fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<GenerateResult> {
        let response = self.send_request(prompt, &options, false).await?;
        let body = response.text().await.map_err(|e| AiError::Transport {
            message: e.to_string(),
            source: Some(Box::new(e)),
        })?;

        let api_response: crate::api_types::MessagesResponse =
            serde_json::from_str(&body).map_err(|e| {
                AiError::Serialization(format!("Failed to parse Anthropic response: {e}"))
            })?;

        Ok(convert::convert_response(api_response))
    }

    async fn stream(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<AiStream> {
        let response = self.send_request(prompt, &options, true).await?;
        Ok(stream_parser::parse_stream(response))
    }
}
