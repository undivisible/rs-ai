use secrecy::SecretString;

use crate::model::GeminiModel;

// ── Well-known model identifiers ──

/// Gemini 2.5 Pro — most capable reasoning model.
pub const GEMINI_25_PRO: &str = "gemini-2.5-pro";
/// Gemini 2.5 Flash — best price/performance.
pub const GEMINI_25_FLASH: &str = "gemini-2.5-flash";
/// Gemini 2.5 Flash Lite — fastest and cheapest.
pub const GEMINI_25_FLASH_LITE: &str = "gemini-2.5-flash-lite";
/// Gemini 3.1 Pro Preview — latest reasoning + multimodal (preview).
pub const GEMINI_31_PRO_PREVIEW: &str = "gemini-3.1-pro-preview";
/// Gemini 3 Flash — frontier-class at low cost (preview).
pub const GEMINI_3_FLASH: &str = "gemini-3-flash";
/// Gemini 3.1 Flash Live Preview — real-time audio-to-audio dialogue.
pub const GEMINI_31_FLASH_LIVE: &str = "gemini-3.1-flash-live-preview";
/// Gemini Embedding 2 Preview — first multimodal embedding model.
pub const GEMINI_EMBEDDING_2: &str = "gemini-embedding-2-preview";

/// Provider for Google Gemini models.
pub struct GeminiProvider {
    api_key: SecretString,
}

impl GeminiProvider {
    /// Create a new Gemini provider with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: SecretString::from(api_key.into()),
        }
    }

    /// Get Gemini 2.5 Pro (most capable reasoning).
    pub fn gemini_pro(&self) -> GeminiModel {
        self.model(GEMINI_25_PRO)
    }

    /// Get Gemini 2.5 Flash (best price/performance).
    pub fn gemini_flash(&self) -> GeminiModel {
        self.model(GEMINI_25_FLASH)
    }

    /// Get Gemini 2.5 Flash Lite (fastest/cheapest).
    pub fn gemini_flash_lite(&self) -> GeminiModel {
        self.model(GEMINI_25_FLASH_LITE)
    }

    /// Get Gemini 3.1 Pro Preview (latest preview).
    pub fn gemini_31_pro(&self) -> GeminiModel {
        self.model(GEMINI_31_PRO_PREVIEW)
    }

    /// Get Gemini 3 Flash (frontier-class preview).
    pub fn gemini_3_flash(&self) -> GeminiModel {
        self.model(GEMINI_3_FLASH)
    }

    /// Get a Gemini model by its model ID.
    pub fn model(&self, model_id: &str) -> GeminiModel {
        use secrecy::ExposeSecret;
        GeminiModel::new(self.api_key.expose_secret(), model_id)
    }

    /// Fetch the list of models from the Gemini API.
    ///
    /// Calls `GET /v1beta/models?key=...` and returns model names.
    pub async fn list_remote_models(&self) -> rusty_ai::AiResult<Vec<String>> {
        use secrecy::ExposeSecret;
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            self.api_key.expose_secret()
        );
        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| rusty_ai::AiError::Transport {
                message: e.to_string(),
                source: Some(Box::new(e)),
            })?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(rusty_ai::AiError::ProviderError {
                provider: "gemini".into(),
                status: None,
                message: body,
            });
        }

        #[derive(serde::Deserialize)]
        struct ListModelsResponse {
            models: Vec<ModelEntry>,
        }
        #[derive(serde::Deserialize)]
        struct ModelEntry {
            name: String,
        }

        let list: ListModelsResponse = resp
            .json()
            .await
            .map_err(|e| rusty_ai::AiError::Serialization(e.to_string()))?;

        Ok(list
            .models
            .into_iter()
            .map(|m| m.name.strip_prefix("models/").unwrap_or(&m.name).to_string())
            .collect())
    }
}
