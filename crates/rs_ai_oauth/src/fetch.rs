//! Runtime model discovery and refreshable model catalogs.
//!
//! The parser intentionally accepts both OpenAI-style `data` responses and
//! Gemini-style `models` responses. Providers are allowed to add metadata over
//! time, so unknown fields are ignored while known limits and capabilities are
//! normalized into one model description.

use std::collections::BTreeSet;
use std::time::Duration;

use crate::flow::{OAuthError, OAuthProvider};
use serde::{Deserialize, Serialize};

/// Limits advertised by a remote model catalog.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelLimits {
    /// Maximum total context window, in tokens.
    pub context_window: Option<u64>,
    /// Maximum input tokens, when separately advertised.
    pub max_input_tokens: Option<u64>,
    /// Maximum generated output tokens.
    pub max_output_tokens: Option<u64>,
}

/// Pricing advertised by a remote model catalog.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Price per input token.
    pub input_per_token: Option<f64>,
    /// Price per output token.
    pub output_per_token: Option<f64>,
    /// Fixed price per request.
    pub request: Option<f64>,
    /// Price per image input.
    pub image_input: Option<f64>,
    /// Price per internal reasoning token.
    pub reasoning: Option<f64>,
    /// Price per cached input token read.
    pub cache_read: Option<f64>,
    /// Price per cached input token written.
    pub cache_write: Option<f64>,
}

/// Normalized model info returned by OAuth provider discovery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier accepted by the provider.
    pub id: String,
    /// OAuth provider identifier.
    #[serde(default)]
    pub provider: String,
    /// Unix creation timestamp, when supplied.
    #[serde(default)]
    pub created: u64,
    /// Provider ownership field, when supplied.
    #[serde(default)]
    pub owned_by: String,
    /// Human-readable model name.
    #[serde(default)]
    pub display_name: String,
    /// Short provider description.
    #[serde(default)]
    pub description: Option<String>,
    /// Normalized model capabilities (`text_input`, `tool_calling`, etc.).
    #[serde(default)]
    pub capabilities: BTreeSet<String>,
    /// Input modalities such as `text`, `image`, or `audio`.
    #[serde(default)]
    pub input_modalities: Vec<String>,
    /// Output modalities such as `text`, `image`, or `audio`.
    #[serde(default)]
    pub output_modalities: Vec<String>,
    /// Provider-specific parameters supported by this model.
    #[serde(default)]
    pub supported_parameters: BTreeSet<String>,
    /// Context and output limits.
    #[serde(default)]
    pub limits: ModelLimits,
    /// Pricing information, when supplied.
    #[serde(default)]
    pub pricing: Option<ModelPricing>,
    /// Deprecation or expiration date, when supplied.
    #[serde(default)]
    pub expiration_date: Option<String>,
}

impl ModelInfo {
    fn new(id: String) -> Self {
        let mut capabilities = BTreeSet::new();
        capabilities.insert("text_input".to_string());
        capabilities.insert("text_output".to_string());
        capabilities.insert("streaming".to_string());
        Self {
            display_name: id.clone(),
            id,
            provider: String::new(),
            created: 0,
            owned_by: String::new(),
            description: None,
            capabilities,
            input_modalities: vec!["text".to_string()],
            output_modalities: vec!["text".to_string()],
            supported_parameters: BTreeSet::new(),
            limits: ModelLimits::default(),
            pricing: None,
            expiration_date: None,
        }
    }
}

/// A diff produced by refreshing a [`ModelCatalog`].
#[derive(Debug, Clone, Default)]
pub struct ModelCatalogUpdate {
    /// Models newly advertised by the provider.
    pub added: Vec<ModelInfo>,
    /// Existing models whose metadata changed.
    pub updated: Vec<ModelInfo>,
    /// Models removed from the provider's latest snapshot.
    pub removed: Vec<ModelInfo>,
    /// Monotonically increasing catalog generation.
    pub generation: u64,
}

/// A refreshable model list for one OAuth provider.
///
/// This keeps the previous snapshot and reports additions/removals, which is
/// useful for model pickers and long-running consumers that should notice new
/// provider models without restarting.
#[derive(Debug, Clone)]
pub struct ModelCatalog {
    provider: OAuthProvider,
    models: Vec<ModelInfo>,
    generation: u64,
}

/// Models discovered for one logged-in OAuth provider.
#[derive(Debug, Clone)]
pub struct ProviderModels {
    /// Provider whose token was used.
    pub provider: OAuthProvider,
    /// Latest model snapshot.
    pub models: Vec<ModelInfo>,
}

impl ModelCatalog {
    /// Create an empty catalog for a provider.
    pub fn new(provider: OAuthProvider) -> Self {
        Self {
            provider,
            models: Vec::new(),
            generation: 0,
        }
    }

    /// Provider represented by this catalog.
    pub fn provider(&self) -> OAuthProvider {
        self.provider
    }

    /// Current model snapshot.
    pub fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    /// Snapshot generation, incremented after every refresh.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Refresh synchronously and return the model diff.
    pub fn refresh(&mut self, token: &str) -> Result<ModelCatalogUpdate, OAuthError> {
        let models = fetch_models(self.provider, token)?;
        Ok(self.replace(models))
    }

    /// Refresh asynchronously and return the model diff.
    pub async fn refresh_async(&mut self, token: &str) -> Result<ModelCatalogUpdate, OAuthError> {
        let models = fetch_models_async(self.provider, token).await?;
        Ok(self.replace(models))
    }

    fn replace(&mut self, models: Vec<ModelInfo>) -> ModelCatalogUpdate {
        let mut next = Vec::new();
        for model in models {
            if !next
                .iter()
                .any(|existing: &ModelInfo| existing.id == model.id)
            {
                next.push(model);
            }
        }

        let added = next
            .iter()
            .filter(|model| !self.models.iter().any(|old| old.id == model.id))
            .cloned()
            .collect();
        let updated = next
            .iter()
            .filter(|model| {
                self.models
                    .iter()
                    .find(|old| old.id == model.id)
                    .is_some_and(|old| old != *model)
            })
            .cloned()
            .collect();
        let removed = self
            .models
            .iter()
            .filter(|old| !next.iter().any(|model| model.id == old.id))
            .cloned()
            .collect();

        self.models = next;
        self.generation = self.generation.saturating_add(1);
        ModelCatalogUpdate {
            added,
            updated,
            removed,
            generation: self.generation,
        }
    }
}

/// Fetch available models synchronously.
pub fn fetch_models(provider: OAuthProvider, token: &str) -> Result<Vec<ModelInfo>, OAuthError> {
    let url = format!("{}/models", provider.api_base().trim_end_matches('/'));
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/json")
        .timeout(Duration::from_secs(10))
        .send()
        .map_err(|e| OAuthError::Network(format!("models request failed: {e}")))?;

    let status = response.status();
    let value: serde_json::Value = response
        .json()
        .map_err(|e| OAuthError::Network(format!("models parse failed: {e}")))?;
    if !status.is_success() {
        return Err(OAuthError::Network(format!(
            "models request returned status {status}: {}",
            error_detail(&value)
        )));
    }
    let mut models = parse_models(&value)?;
    for model in &mut models {
        model.provider = provider.name().to_string();
    }
    Ok(models)
}

/// Fetch available models asynchronously.
pub async fn fetch_models_async(
    provider: OAuthProvider,
    token: &str,
) -> Result<Vec<ModelInfo>, OAuthError> {
    let url = format!("{}/models", provider.api_base().trim_end_matches('/'));
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/json")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| OAuthError::Network(format!("models request failed: {e}")))?;

    let status = response.status();
    let value: serde_json::Value = response
        .json()
        .await
        .map_err(|e| OAuthError::Network(format!("models parse failed: {e}")))?;
    if !status.is_success() {
        return Err(OAuthError::Network(format!(
            "models request returned status {status}: {}",
            error_detail(&value)
        )));
    }
    let mut models = parse_models(&value)?;
    for model in &mut models {
        model.provider = provider.name().to_string();
    }
    Ok(models)
}

/// Fetch model snapshots for every provider with stored credentials.
///
/// This is the convenient consumer API for a model picker: it discovers the
/// currently logged-in provider set from the shared credential store and then
/// asks each provider for its current models. A failed provider returns an
/// error rather than silently hiding a stale or incomplete catalog.
pub fn fetch_logged_in_models() -> Result<Vec<ProviderModels>, OAuthError> {
    let mut result = Vec::new();
    for provider in crate::credentials::logged_in_providers() {
        let tokens = crate::credentials::load(&provider).ok_or_else(|| {
            OAuthError::Auth(format!("credentials disappeared for {}", provider.name()))
        })?;
        result.push(ProviderModels {
            provider,
            models: fetch_models(provider, &tokens.access_token)?,
        });
    }
    Ok(result)
}

/// Async counterpart to [`fetch_logged_in_models`].
pub async fn fetch_logged_in_models_async() -> Result<Vec<ProviderModels>, OAuthError> {
    let mut result = Vec::new();
    for provider in crate::credentials::logged_in_providers() {
        let tokens = load_tokens_for_discovery(provider).await?;
        result.push(ProviderModels {
            provider,
            models: fetch_models_async(provider, &tokens.access_token).await?,
        });
    }
    Ok(result)
}

/// Load a token for background discovery, refreshing it when possible.
///
/// Model pickers are often long-lived and may be the first consumer to touch
/// a credential after it expires. Persisting the refreshed token here keeps
/// the shared store useful to the next consumer as well.
async fn load_tokens_for_discovery(
    provider: OAuthProvider,
) -> Result<crate::flow::OAuthTokens, OAuthError> {
    let tokens = crate::credentials::load(&provider).ok_or_else(|| {
        OAuthError::Auth(format!("credentials disappeared for {}", provider.name()))
    })?;
    if !crate::credentials::is_expired(&tokens) {
        return Ok(tokens);
    }

    let refreshed = crate::flow::refresh_oauth_token(provider, &tokens).await?;
    crate::credentials::save(&provider, &refreshed).map_err(|error| {
        OAuthError::Auth(format!(
            "refreshed {} token could not be saved: {error}",
            provider.name()
        ))
    })?;
    Ok(refreshed)
}

fn parse_models(value: &serde_json::Value) -> Result<Vec<ModelInfo>, OAuthError> {
    if let Some(data) = value.get("data").and_then(serde_json::Value::as_array) {
        return Ok(data.iter().filter_map(parse_openai_model).collect());
    }
    if let Some(models) = value.get("models").and_then(serde_json::Value::as_array) {
        return Ok(models.iter().filter_map(parse_gemini_model).collect());
    }
    Err(OAuthError::Network(
        "models response contained neither a data nor models array".into(),
    ))
}

fn parse_openai_model(value: &serde_json::Value) -> Option<ModelInfo> {
    let id = value.get("id")?.as_str()?.to_string();
    let mut info = ModelInfo::new(id.clone());
    info.display_name = value
        .get("name")
        .or_else(|| value.get("display_name"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&id)
        .to_string();
    info.created = value.get("created").and_then(as_u64).unwrap_or(0);
    info.owned_by = value
        .get("owned_by")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    info.description = value
        .get("description")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    info.expiration_date = value
        .get("expiration_date")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);

    if let Some(architecture) = value.get("architecture") {
        info.input_modalities = string_array(architecture.get("input_modalities"));
        info.output_modalities = string_array(architecture.get("output_modalities"));
        if !info.input_modalities.is_empty() && !info.input_modalities.iter().any(|m| m == "text") {
            info.capabilities.remove("text_input");
        }
        if !info.output_modalities.is_empty() && !info.output_modalities.iter().any(|m| m == "text")
        {
            info.capabilities.remove("text_output");
            info.capabilities.remove("streaming");
        }
        apply_modality_capabilities(&mut info);
    }
    if let Some(parameters) = value
        .get("supported_parameters")
        .and_then(serde_json::Value::as_array)
    {
        for parameter in parameters.iter().filter_map(serde_json::Value::as_str) {
            info.supported_parameters.insert(parameter.to_string());
            match parameter {
                "tools" | "tool_choice" => {
                    info.capabilities.insert("tool_calling".into());
                }
                "structured_outputs" | "response_format" => {
                    info.capabilities.insert("structured_output".into());
                }
                "reasoning" | "include_reasoning" => {
                    info.capabilities.insert("extended_thinking".into());
                }
                _ => {}
            }
        }
    }

    if let Some(top_provider) = value.get("top_provider") {
        info.limits.context_window = first_u64(
            top_provider.get("context_length"),
            value.get("context_length"),
        );
        info.limits.max_output_tokens = top_provider.get("max_completion_tokens").and_then(as_u64);
    } else {
        info.limits.context_window = value.get("context_length").and_then(as_u64);
    }
    info.pricing = parse_pricing(value.get("pricing"));
    Some(info)
}

fn parse_gemini_model(value: &serde_json::Value) -> Option<ModelInfo> {
    let raw_name = value.get("name")?.as_str()?;
    let id = raw_name
        .strip_prefix("models/")
        .unwrap_or(raw_name)
        .to_string();
    let mut info = ModelInfo::new(id);
    info.display_name = value
        .get("displayName")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&info.id)
        .to_string();
    info.description = value
        .get("description")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    info.limits.max_input_tokens = value.get("inputTokenLimit").and_then(as_u64);
    info.limits.context_window = info.limits.max_input_tokens;
    info.limits.max_output_tokens = value.get("outputTokenLimit").and_then(as_u64);

    if let Some(methods) = value
        .get("supportedGenerationMethods")
        .and_then(serde_json::Value::as_array)
    {
        for method in methods.iter().filter_map(serde_json::Value::as_str) {
            info.supported_parameters.insert(method.to_string());
            match method {
                "generateContent" | "countTokens" => {}
                "embedContent" => {
                    info.capabilities.insert("embeddings".into());
                }
                _ => {}
            }
        }
        if info.capabilities.contains("embeddings")
            && !info.supported_parameters.contains("generateContent")
        {
            info.capabilities.remove("text_output");
        }
    }
    if value.get("thinking").and_then(serde_json::Value::as_bool) == Some(true) {
        info.capabilities.insert("extended_thinking".into());
    }
    Some(info)
}

fn apply_modality_capabilities(info: &mut ModelInfo) {
    for modality in &info.input_modalities {
        match modality.as_str() {
            "image" => {
                info.capabilities.insert("image_input".into());
            }
            "audio" => {
                info.capabilities.insert("audio_input".into());
            }
            "video" => {
                info.capabilities.insert("video_input".into());
            }
            _ => {}
        }
    }
    for modality in &info.output_modalities {
        match modality.as_str() {
            "image" => {
                info.capabilities.insert("image_output".into());
                info.capabilities.insert("image_generation".into());
            }
            "audio" => {
                info.capabilities.insert("audio_output".into());
            }
            "video" => {
                info.capabilities.insert("video_generation".into());
            }
            _ => {}
        }
    }
}

fn parse_pricing(value: Option<&serde_json::Value>) -> Option<ModelPricing> {
    let object = value?.as_object()?;
    let pricing = ModelPricing {
        input_per_token: number(object.get("prompt")),
        output_per_token: number(object.get("completion")),
        request: number(object.get("request")),
        image_input: number(object.get("image")),
        reasoning: number(object.get("internal_reasoning")),
        cache_read: number(object.get("input_cache_read")),
        cache_write: number(object.get("input_cache_write")),
    };
    Some(pricing)
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

fn as_u64(value: &serde_json::Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_f64().filter(|v| *v >= 0.0).map(|v| v as u64))
        .or_else(|| value.as_str()?.parse().ok())
}

fn first_u64(first: Option<&serde_json::Value>, second: Option<&serde_json::Value>) -> Option<u64> {
    first.and_then(as_u64).or_else(|| second.and_then(as_u64))
}

fn number(value: Option<&serde_json::Value>) -> Option<f64> {
    value.and_then(|v| v.as_f64().or_else(|| v.as_str()?.parse().ok()))
}

fn error_detail(value: &serde_json::Value) -> String {
    value
        .get("error")
        .and_then(|error| error.get("message").or(Some(error)))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown provider error")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openrouter_metadata_maps_capabilities_and_limits() {
        let json = serde_json::json!({
            "data": [{
                "id": "anthropic/claude-sonnet-4",
                "name": "Claude Sonnet 4",
                "created": 1700000000,
                "owned_by": "anthropic",
                "context_length": 200000,
                "architecture": {
                    "input_modalities": ["text", "image"],
                    "output_modalities": ["text"]
                },
                "top_provider": {"context_length": 180000, "max_completion_tokens": 64000},
                "pricing": {"prompt": "0.000003", "completion": "0.000015"},
                "supported_parameters": ["tools", "structured_outputs", "reasoning"]
            }]
        });

        let models = parse_models(&json).unwrap();
        let model = &models[0];
        assert_eq!(model.limits.context_window, Some(180_000));
        assert_eq!(model.limits.max_output_tokens, Some(64_000));
        assert!(model.capabilities.contains("image_input"));
        assert!(model.capabilities.contains("tool_calling"));
        assert!(model.capabilities.contains("structured_output"));
        assert_eq!(
            model.pricing.as_ref().unwrap().input_per_token,
            Some(0.000003)
        );
    }

    #[test]
    fn gemini_metadata_maps_native_limits() {
        let json = serde_json::json!({
            "models": [{
                "name": "models/gemini-2.5-flash",
                "displayName": "Gemini 2.5 Flash",
                "inputTokenLimit": 1048576,
                "outputTokenLimit": 65536,
                "supportedGenerationMethods": ["generateContent", "countTokens"],
                "thinking": true
            }]
        });

        let model = &parse_models(&json).unwrap()[0];
        assert_eq!(model.id, "gemini-2.5-flash");
        assert_eq!(model.limits.context_window, Some(1_048_576));
        assert_eq!(model.limits.max_output_tokens, Some(65_536));
        assert!(model.capabilities.contains("extended_thinking"));
    }

    #[test]
    fn catalog_reports_added_updated_and_removed_models() {
        let mut catalog = ModelCatalog::new(OAuthProvider::Xai);
        let a = ModelInfo::new("a".into());
        let b = ModelInfo::new("b".into());
        let first = catalog.replace(vec![a.clone(), b.clone()]);
        assert_eq!(first.added.len(), 2);

        let mut changed = a.clone();
        changed.description = Some("new".into());
        let second = catalog.replace(vec![changed.clone()]);
        assert_eq!(second.updated, vec![changed]);
        assert_eq!(second.removed, vec![b]);
        assert_eq!(second.generation, 2);
    }

    #[test]
    fn parse_models_errors_when_neither_data_nor_models() {
        let err = parse_models(&serde_json::json!({})).unwrap_err();
        assert!(err.to_string().contains("neither a data nor models"));
        let err = parse_models(&serde_json::json!({ "data": {} })).unwrap_err();
        assert!(err.to_string().contains("neither a data nor models"));
    }

    #[test]
    fn parse_models_prefers_data_array_over_models() {
        let json = serde_json::json!({
            "data": [{ "id": "from-data" }],
            "models": [{ "name": "models/from-gemini" }]
        });
        let models = parse_models(&json).unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "from-data");
    }

    #[test]
    fn parse_models_skips_entries_without_id() {
        let json = serde_json::json!({
            "data": [{}, { "id": "ok" }, { "name": "not-an-id" }]
        });
        let models = parse_models(&json).unwrap();
        assert_eq!(
            models.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            ["ok"]
        );
    }

    #[test]
    fn openai_created_string_and_float_are_accepted() {
        let json = serde_json::json!({
            "data": [
                { "id": "string-created", "created": "1700000000" },
                { "id": "float-created", "created": 1700000000.0 }
            ]
        });
        let models = parse_models(&json).unwrap();
        assert_eq!(models[0].created, 1_700_000_000);
        assert_eq!(models[1].created, 1_700_000_000);
    }

    #[test]
    fn video_input_and_output_map_to_capabilities() {
        let json = serde_json::json!({
            "data": [{
                "id": "video-model",
                "architecture": {
                    "input_modalities": ["text", "video"],
                    "output_modalities": ["text", "video"]
                }
            }]
        });
        let model = &parse_models(&json).unwrap()[0];
        assert!(model.capabilities.contains("video_input"));
        assert!(model.capabilities.contains("video_generation"));
        assert!(model.capabilities.contains("text_output"));
        assert!(model.capabilities.contains("streaming"));
    }

    #[test]
    fn image_only_output_drops_streaming() {
        let json = serde_json::json!({
            "data": [{
                "id": "image-model",
                "architecture": {
                    "input_modalities": ["text"],
                    "output_modalities": ["image"]
                }
            }]
        });
        let model = &parse_models(&json).unwrap()[0];
        assert!(model.capabilities.contains("image_generation"));
        assert!(!model.capabilities.contains("streaming"));
        assert!(!model.capabilities.contains("text_output"));
    }

    #[test]
    fn gemini_embed_only_drops_text_output() {
        let json = serde_json::json!({
            "models": [{
                "name": "models/text-embedding-004",
                "supportedGenerationMethods": ["embedContent"]
            }]
        });
        let model = &parse_models(&json).unwrap()[0];
        assert!(model.capabilities.contains("embeddings"));
        assert!(!model.capabilities.contains("text_output"));
    }

    #[test]
    fn gemini_embed_plus_generate_keeps_text_output() {
        let json = serde_json::json!({
            "models": [{
                "name": "models/gemini-embed-and-chat",
                "supportedGenerationMethods": ["generateContent", "embedContent"]
            }]
        });
        let model = &parse_models(&json).unwrap()[0];
        assert!(model.capabilities.contains("embeddings"));
        assert!(model.capabilities.contains("text_output"));
        assert!(model.capabilities.contains("streaming"));
    }

    #[test]
    fn gemini_name_without_models_prefix_is_kept() {
        let json = serde_json::json!({
            "models": [{ "name": "gemini-2.5-flash" }]
        });
        assert_eq!(parse_models(&json).unwrap()[0].id, "gemini-2.5-flash");
    }

    #[test]
    fn catalog_replace_keeps_first_duplicate_id() {
        let mut catalog = ModelCatalog::new(OAuthProvider::Xai);
        let mut first = ModelInfo::new("dup".into());
        first.description = Some("keep".into());
        let mut second = ModelInfo::new("dup".into());
        second.description = Some("drop".into());
        catalog.replace(vec![first.clone(), second]);
        assert_eq!(catalog.models().len(), 1);
        assert_eq!(catalog.models()[0].description.as_deref(), Some("keep"));
    }

    #[test]
    fn catalog_noop_refresh_still_bumps_generation() {
        let mut catalog = ModelCatalog::new(OAuthProvider::Xai);
        let model = ModelInfo::new("a".into());
        catalog.replace(vec![model.clone()]);
        let second = catalog.replace(vec![model]);
        assert!(second.added.is_empty());
        assert!(second.updated.is_empty());
        assert!(second.removed.is_empty());
        assert_eq!(second.generation, 2);
    }
}
