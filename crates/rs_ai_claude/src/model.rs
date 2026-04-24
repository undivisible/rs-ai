use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};

use rs_ai_traits::capability::{Capability, CapabilitySet};
use rs_ai_traits::error::{AiError, AiResult};
use rs_ai_traits::model::{GenerateOptions, LanguageModel};
use rs_ai_traits::prompt::Prompt;
use rs_ai_traits::stream::AiStream;
use rs_ai_traits::structured::GenerateResult;

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
    cache_config: Option<rs_ai_cache::CacheConfig>,
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
            .with(Capability::ToolCalling)
            .with(Capability::ExtendedThinking)
            .with(Capability::StructuredOutput);

        Self {
            api_key: SecretString::from(api_key.into()),
            model_id: model_id.to_string(),
            capabilities,
            client: reqwest::Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            cache_config: None,
        }
    }

    /// Override the base URL (useful for proxies or testing).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set cache configuration.
    pub fn set_cache(&mut self, config: rs_ai_cache::CacheConfig) -> &mut Self {
        self.cache_config = Some(config);
        self
    }

    /// Set cache configuration with ephemeral or persistent mode.
    pub fn with_cache_ephemeral(mut self, ephemeral: bool) -> Self {
        let mut config = rs_ai_cache::CacheConfig::new();
        if ephemeral {
            config = config.enable_anthropic_ephemeral();
        } else {
            config = config.enable_anthropic_persistent();
        }
        self.cache_config = Some(config);
        self
    }

    /// Send a request to the Anthropic Messages API.
    async fn send_request(
        &self,
        prompt: Prompt,
        options: &GenerateOptions,
        stream: bool,
    ) -> AiResult<reqwest::Response> {
        let request = convert::build_request(
            &self.model_id,
            prompt,
            options,
            stream,
            self.cache_config.as_ref(),
        );

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
            let body_text = match response.text().await {
                Ok(body) => body,
                Err(e) => {
                    tracing::warn!(status = status_code, error = %e, "Failed to read Anthropic error response body");
                    format!("<failed to read response body: {e}>")
                }
            };

            let message = if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body_text)
            {
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

        let api_response: crate::api_types::MessagesResponse = serde_json::from_str(&body)
            .map_err(|e| {
                AiError::Serialization(format!("Failed to parse Anthropic response: {e}"))
            })?;

        Ok(convert::convert_response(api_response))
    }

    async fn stream(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<AiStream> {
        let response = self.send_request(prompt, &options, true).await?;
        Ok(stream_parser::parse_stream(response))
    }
}
