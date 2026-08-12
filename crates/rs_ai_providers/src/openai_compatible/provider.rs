use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use secrecy::ExposeSecret;

use rs_ai_core::{
    AiError, AiResult, Capability, CapabilitySet, EmbeddingModel, LanguageModel, ModelInfo,
    ModelLimits, ModelPricing, Provider,
};

use super::config::OpenAiCompatibleConfig;
use super::model::OpenAiCompatibleModel;

/// A provider backed by an OpenAI-compatible API.
///
/// This is not a trait impl — it is a concrete registry that knows how to
/// create [`OpenAiCompatibleModel`] instances for its registered models.
pub struct OpenAiCompatibleProvider {
    config: OpenAiCompatibleConfig,
    provider_id: String,
    provider_name: String,
    models: Vec<ModelInfo>,
}

impl OpenAiCompatibleProvider {
    /// Create a new provider.
    pub fn new(config: OpenAiCompatibleConfig, id: &str, name: &str) -> Self {
        Self {
            config,
            provider_id: id.to_string(),
            provider_name: name.to_string(),
            models: Vec::new(),
        }
    }

    /// Register a model's metadata. Returns `self` for chaining.
    pub fn with_model_info(mut self, info: ModelInfo) -> Self {
        self.models.push(info);
        self
    }

    /// Return the provider identifier.
    pub fn id(&self) -> &str {
        &self.provider_id
    }

    /// Return the human-readable provider name.
    pub fn name(&self) -> &str {
        &self.provider_name
    }

    /// List the registered model metadata.
    pub fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    /// Create a [`LanguageModel`] for the given model id.
    ///
    /// If the model id matches registered metadata, the capabilities from that
    /// metadata are attached. Otherwise a model with an empty capability set is
    /// returned (OpenAI-compatible APIs typically accept any model string).
    pub fn language_model(&self, model_id: &str) -> Box<dyn LanguageModel> {
        let caps = self
            .models
            .iter()
            .find(|m| m.id == model_id)
            .map(|m| m.capabilities.clone())
            .unwrap_or_else(CapabilitySet::new);

        Box::new(
            OpenAiCompatibleModel::new(self.config.clone(), model_id, &self.provider_id)
                .with_capabilities(caps),
        )
    }

    /// Fetch rich model metadata from this endpoint's `/models` route.
    ///
    /// OpenRouter's response is a superset of the OpenAI model object. The
    /// parser preserves context limits, modalities, supported parameters, and
    /// pricing where they are advertised; sparse OpenAI-compatible responses
    /// still produce usable text models with optional metadata.
    pub async fn list_remote_models(&self) -> AiResult<Vec<ModelInfo>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let auth = format!("Bearer {}", self.config.api_key.expose_secret());
        let auth = HeaderValue::from_str(&auth).map_err(|e| AiError::AuthError {
            message: format!("invalid API key header: {e}"),
        })?;
        headers.insert(AUTHORIZATION, auth);
        for (name, value) in &self.config.default_headers {
            let name = HeaderName::try_from(name.as_str()).map_err(|e| AiError::Transport {
                message: format!("invalid default header name: {e}"),
                source: None,
            })?;
            let value = HeaderValue::from_str(value).map_err(|e| AiError::Transport {
                message: format!("invalid default header value: {e}"),
                source: None,
            })?;
            headers.insert(name, value);
        }
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| AiError::Transport {
                message: e.to_string(),
                source: Some(Box::new(e)),
            })?;
        let url = format!("{}/models", self.config.base_url.trim_end_matches('/'));
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| AiError::Transport {
                message: e.to_string(),
                source: Some(Box::new(e)),
            })?;
        let status = response.status();
        let body = response.text().await.map_err(|e| AiError::Transport {
            message: e.to_string(),
            source: Some(Box::new(e)),
        })?;
        if !status.is_success() {
            return Err(AiError::ProviderError {
                provider: self.provider_id.clone(),
                status: Some(status.as_u16()),
                message: body,
            });
        }
        let value: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| AiError::Serialization(e.to_string()))?;
        let data = value
            .get("data")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| AiError::Serialization("models response has no data array".into()))?;
        Ok(data
            .iter()
            .filter_map(|value| parse_remote_model(value, &self.provider_id))
            .collect())
    }

    /// Replace local model metadata with the endpoint's latest snapshot.
    pub async fn refresh_models(&mut self) -> AiResult<rs_ai_core::ModelRegistryUpdate> {
        let models = self.list_remote_models().await?;
        let mut registry = rs_ai_core::ModelRegistry::new();
        registry.update(std::mem::take(&mut self.models));
        let diff = registry.replace(models.clone());
        self.models = models;
        Ok(diff)
    }
}

#[async_trait]
impl Provider for OpenAiCompatibleProvider {
    fn id(&self) -> &str {
        OpenAiCompatibleProvider::id(self)
    }

    fn name(&self) -> &str {
        OpenAiCompatibleProvider::name(self)
    }

    fn language_model(&self, model_id: &str) -> AiResult<Box<dyn LanguageModel>> {
        Ok(OpenAiCompatibleProvider::language_model(self, model_id))
    }

    fn embedding_model(&self, model_id: &str) -> AiResult<Box<dyn EmbeddingModel>> {
        Err(AiError::UnsupportedCapability {
            capability: "embeddings".into(),
            provider: format!("{} ({model_id})", self.provider_id),
        })
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        self.models.clone()
    }

    async fn fetch_models(&self) -> AiResult<Vec<ModelInfo>> {
        self.list_remote_models().await
    }
}

fn parse_remote_model(value: &serde_json::Value, provider: &str) -> Option<ModelInfo> {
    let id = value.get("id")?.as_str()?.to_string();
    let mut info = ModelInfo {
        id: id.clone(),
        provider: provider.to_string(),
        display_name: value
            .get("name")
            .or_else(|| value.get("display_name"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or(&id)
            .to_string(),
        capabilities: CapabilitySet::new()
            .with(Capability::TextInput)
            .with(Capability::TextOutput)
            .with(Capability::Streaming),
        limits: ModelLimits::default(),
        pricing: None,
        input_modalities: Vec::new(),
        output_modalities: Vec::new(),
        supported_parameters: std::collections::BTreeSet::new(),
        description: value
            .get("description")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        created: value.get("created").and_then(value_u64),
        owned_by: value
            .get("owned_by")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        expiration_date: value
            .get("expiration_date")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
    };

    if let Some(architecture) = value.get("architecture") {
        info.input_modalities = string_array(architecture.get("input_modalities"));
        info.output_modalities = string_array(architecture.get("output_modalities"));
        if !info.input_modalities.is_empty() {
            info.capabilities.remove(&Capability::TextInput);
            if info.input_modalities.iter().any(|m| m == "text") {
                info.capabilities = info.capabilities.with(Capability::TextInput);
            }
        }
        if !info.output_modalities.is_empty() {
            info.capabilities.remove(&Capability::TextOutput);
            if info.output_modalities.iter().any(|m| m == "text") {
                info.capabilities = info.capabilities.with(Capability::TextOutput);
            } else {
                info.capabilities.remove(&Capability::Streaming);
            }
        }
        for modality in &info.input_modalities {
            match modality.as_str() {
                "image" => info.capabilities = info.capabilities.with(Capability::ImageInput),
                "audio" => info.capabilities = info.capabilities.with(Capability::AudioInput),
                _ => {}
            }
        }
        for modality in &info.output_modalities {
            match modality.as_str() {
                "image" => {
                    info.capabilities = info
                        .capabilities
                        .with(Capability::ImageOutput)
                        .with(Capability::ImageGeneration);
                }
                "audio" => info.capabilities = info.capabilities.with(Capability::AudioOutput),
                _ => {}
            }
        }
    }
    if let Some(parameters) = value
        .get("supported_parameters")
        .and_then(serde_json::Value::as_array)
    {
        for parameter in parameters.iter().filter_map(serde_json::Value::as_str) {
            info.supported_parameters.insert(parameter.into());
            match parameter {
                "tools" | "tool_choice" => {
                    info.capabilities = info.capabilities.with(Capability::ToolCalling)
                }
                "structured_outputs" | "response_format" => {
                    info.capabilities = info.capabilities.with(Capability::StructuredOutput)
                }
                "reasoning" | "include_reasoning" => {
                    info.capabilities = info.capabilities.with(Capability::ExtendedThinking)
                }
                _ => {}
            }
        }
    }

    let top_provider = value.get("top_provider");
    info.limits.context_window = top_provider
        .and_then(|v| v.get("context_length"))
        .and_then(value_u64)
        .or_else(|| value.get("context_length").and_then(value_u64));
    info.limits.max_output_tokens = top_provider
        .and_then(|v| v.get("max_completion_tokens"))
        .and_then(value_u64);
    info.pricing = parse_pricing(value.get("pricing"));
    Some(info)
}

fn string_array(value: Option<&serde_json::Value>) -> Vec<String> {
    value
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn value_u64(value: &serde_json::Value) -> Option<u64> {
    value.as_u64().or_else(|| value.as_str()?.parse().ok())
}

fn number(value: Option<&serde_json::Value>) -> Option<f64> {
    value.and_then(|value| value.as_f64().or_else(|| value.as_str()?.parse().ok()))
}

fn parse_pricing(value: Option<&serde_json::Value>) -> Option<ModelPricing> {
    let object = value?.as_object()?;
    Some(ModelPricing {
        input_per_token: number(object.get("prompt")),
        output_per_token: number(object.get("completion")),
        request: number(object.get("request")),
        image_input: number(object.get("image")),
        reasoning: number(object.get("internal_reasoning")),
        cache_read: number(object.get("input_cache_read")),
        cache_write: number(object.get("input_cache_write")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai_compatible::OpenAiCompatibleConfig;

    #[test]
    fn parses_openrouter_model_metadata() {
        let json = serde_json::json!({
            "id": "openai/gpt-4o-mini",
            "name": "GPT-4o Mini",
            "created": 1700000000,
            "owned_by": "openai",
            "architecture": {
                "input_modalities": ["text", "image"],
                "output_modalities": ["text"]
            },
            "context_length": 128000,
            "top_provider": {"context_length": 128000, "max_completion_tokens": 16384},
            "pricing": {"prompt": "0.00000015", "completion": "0.0000006"},
            "supported_parameters": ["tools", "structured_outputs"]
        });
        let model = parse_remote_model(&json, "openrouter").unwrap();
        assert_eq!(model.limits.context_window, Some(128000));
        assert_eq!(model.limits.max_output_tokens, Some(16384));
        assert!(model.capabilities.has(&Capability::ImageInput));
        assert!(model.capabilities.has(&Capability::ToolCalling));
        assert!(model.capabilities.has(&Capability::StructuredOutput));
        assert_eq!(model.pricing.unwrap().input_per_token, Some(0.00000015));

        let provider = OpenAiCompatibleProvider::new(
            OpenAiCompatibleConfig::new("https://example.com/v1", "key"),
            "openrouter",
            "OpenRouter",
        );
        assert!(provider.models().is_empty());
    }
}
