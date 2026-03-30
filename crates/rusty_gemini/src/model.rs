use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};

use rusty_ai::{
    AiError, AiResult, AiStream, Capability, CapabilitySet, GenerateOptions, GenerateResult,
    LanguageModel, Prompt,
};

use crate::api_types::GenerateContentRequest;
use crate::convert::{build_request, response_to_result};
use crate::stream_parser;

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/// A Google Gemini language model.
pub struct GeminiModel {
    api_key: SecretString,
    model_id: String,
    capabilities: CapabilitySet,
    client: reqwest::Client,
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
            .with(Capability::StructuredOutput);

        Self {
            api_key: SecretString::from(api_key.into()),
            model_id: model_id.to_string(),
            capabilities,
            client: reqwest::Client::new(),
        }
    }

    fn generate_url(&self) -> String {
        format!(
            "{}/{}:generateContent?key={}",
            BASE_URL,
            self.model_id,
            self.api_key.expose_secret()
        )
    }

    fn stream_url(&self) -> String {
        format!(
            "{}/{}:streamGenerateContent?key={}&alt=sse",
            BASE_URL,
            self.model_id,
            self.api_key.expose_secret()
        )
    }

    fn build_api_request(
        &self,
        prompt: Prompt,
        options: &GenerateOptions,
    ) -> GenerateContentRequest {
        let (contents, system_instruction, generation_config, tools, tool_config) =
            build_request(prompt, options);

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

    async fn generate(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> AiResult<GenerateResult> {
        let request_body = self.build_api_request(prompt, &options);

        let response = self
            .client
            .post(&self.generate_url())
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
            let body = response.text().await.unwrap_or_default();
            return Err(AiError::ProviderError {
                provider: "gemini".to_string(),
                status: Some(status.as_u16()),
                message: body,
            });
        }

        let api_response: crate::api_types::GenerateContentResponse =
            response.json().await.map_err(|e| AiError::Serialization(e.to_string()))?;

        Ok(response_to_result(api_response, &self.model_id))
    }

    async fn stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> AiResult<AiStream> {
        let request_body = self.build_api_request(prompt, &options);

        let response = self
            .client
            .post(&self.stream_url())
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
            let body = response.text().await.unwrap_or_default();
            return Err(AiError::ProviderError {
                provider: "gemini".to_string(),
                status: Some(status.as_u16()),
                message: body,
            });
        }

        Ok(stream_parser::parse_stream(response))
    }
}
