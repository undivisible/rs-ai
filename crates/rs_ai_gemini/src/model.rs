use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};

use rs_ai_ai::{
    AiError, AiResult, AiStream, Capability, CapabilitySet, GenerateOptions, GenerateResult,
    LanguageModel, Prompt,
};

use crate::api_types::GenerateContentRequest;
use crate::convert::{build_request, response_to_result, GeminiRequestParts};
use crate::stream_parser;

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/// A Google Gemini language model.
pub struct GeminiModel {
    api_key: SecretString,
    model_id: String,
    capabilities: CapabilitySet,
    client: reqwest::Client,
    base_url: String,
    cache_config: Option<rs_ai_cache::CacheConfig>,
}

impl GeminiModel {
    /// Create a new Gemini model instance.
    pub fn new(api_key: impl Into<String>, model_id: &str) -> Self {
        let capabilities = CapabilitySet::new()
            .with(Capability::TextInput)
            .with(Capability::TextOutput)
            .with(Capability::ImageInput)
            .with(Capability::Streaming)
            .with(Capability::ToolCalling)
            .with(Capability::StructuredOutput)
            .with(Capability::ExtendedThinking)
            .with(Capability::VideoInput)
            .with(Capability::AudioInput);

        Self {
            api_key: SecretString::from(api_key.into()),
            model_id: model_id.to_string(),
            capabilities,
            client: reqwest::Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            cache_config: None,
        }
    }

    /// Override the base URL (e.g. for Cloudflare AI Gateway).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set cache configuration.
    pub fn set_cache(&mut self, config: rs_ai_cache::CacheConfig) -> &mut Self {
        self.cache_config = Some(config);
        self
    }

    /// Set cache configuration with a cache key.
    pub fn with_cache_key(mut self, key: impl Into<String>) -> Self {
        let mut config = rs_ai_cache::CacheConfig::new();
        config = config.with_gemini_cache_key(key);
        self.cache_config = Some(config);
        self
    }

    fn generate_url(&self) -> String {
        let mut url = format!(
            "{}/{}:generateContent?key={}",
            self.base_url,
            self.model_id,
            self.api_key.expose_secret()
        );

        if let Some(config) = &self.cache_config {
            if let Some(cache_key) = &config.gemini_cache_key {
                url.push_str(&format!("&cacheKey={}", urlencoding::encode(cache_key)));
            }
        }

        url
    }

    fn stream_url(&self) -> String {
        let mut url = format!(
            "{}/{}:streamGenerateContent?key={}&alt=sse",
            self.base_url,
            self.model_id,
            self.api_key.expose_secret()
        );

        if let Some(config) = &self.cache_config {
            if let Some(cache_key) = &config.gemini_cache_key {
                url.push_str(&format!("&cacheKey={}", urlencoding::encode(cache_key)));
            }
        }

        url
    }

    fn build_api_request(
        &self,
        prompt: Prompt,
        options: &GenerateOptions,
    ) -> GenerateContentRequest {
        let GeminiRequestParts {
            contents,
            system_instruction,
            generation_config,
            tools,
            tool_config,
        } = build_request(prompt, options);

        GenerateContentRequest {
            contents,
            system_instruction,
            generation_config,
            tools,
            tool_config,
        }
    }
}

#[async_trait]
impl LanguageModel for GeminiModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "gemini"
    }

    fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<GenerateResult> {
        let request_body = self.build_api_request(prompt, &options);

        let response = self
            .client
            .post(self.generate_url())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AiError::Transport {
                message: e.to_string(),
                source: Some(Box::new(e)),
            })?;

        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = match response.text().await {
                Ok(body) => body,
                Err(e) => {
                    tracing::warn!(status = status_code, error = %e, "Failed to read Gemini error response body");
                    format!("<failed to read response body: {e}>")
                }
            };
            return Err(AiError::ProviderError {
                provider: "gemini".to_string(),
                status: Some(status_code),
                message: body,
            });
        }

        let api_response: crate::api_types::GenerateContentResponse = response
            .json()
            .await
            .map_err(|e| AiError::Serialization(e.to_string()))?;

        Ok(response_to_result(api_response, &self.model_id))
    }

    async fn stream(&self, prompt: Prompt, options: GenerateOptions) -> AiResult<AiStream> {
        let request_body = self.build_api_request(prompt, &options);

        let response = self
            .client
            .post(self.stream_url())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AiError::Transport {
                message: e.to_string(),
                source: Some(Box::new(e)),
            })?;

        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = match response.text().await {
                Ok(body) => body,
                Err(e) => {
                    tracing::warn!(status = status_code, error = %e, "Failed to read Gemini error response body");
                    format!("<failed to read response body: {e}>")
                }
            };
            return Err(AiError::ProviderError {
                provider: "gemini".to_string(),
                status: Some(status_code),
                message: body,
            });
        }

        Ok(stream_parser::parse_stream(response))
    }
}
