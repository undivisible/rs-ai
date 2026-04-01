use secrecy::SecretString;

use crate::model::GeminiModel;

// ── Latest model aliases ──

/// Convenience alias. Use `fetch_models()` or pass any model ID string to `model()`.
pub const GEMINI_PRO_LATEST: &str = "gemini-2.5-pro";
/// Convenience alias. Use `fetch_models()` or pass any model ID string to `model()`.
pub const GEMINI_FLASH_LATEST: &str = "gemini-2.5-flash";
/// Convenience alias. Use `fetch_models()` or pass any model ID string to `model()`.
pub const GEMINI_FLASH_LITE_LATEST: &str = "gemini-2.5-flash-lite";
/// Convenience alias. Use `fetch_models()` or pass any model ID string to `model()`.
pub const GEMINI_PRO_PREVIEW_LATEST: &str = "gemini-3.1-pro-preview";
/// Convenience alias. Use `fetch_models()` or pass any model ID string to `model()`.
pub const GEMINI_3_FLASH_LATEST: &str = "gemini-3-flash";
/// Convenience alias. Use `fetch_models()` or pass any model ID string to `model()`.
pub const GEMINI_FLASH_LIVE_LATEST: &str = "gemini-3.1-flash-live-preview";
/// Convenience alias. Use `fetch_models()` or pass any model ID string to `model()`.
pub const GEMINI_EMBEDDING_LATEST: &str = "gemini-embedding-2-preview";

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
        self.model(GEMINI_PRO_LATEST)
    }

    /// Get Gemini 2.5 Flash (best price/performance).
    pub fn gemini_flash(&self) -> GeminiModel {
        self.model(GEMINI_FLASH_LATEST)
    }

    /// Get Gemini 2.5 Flash Lite (fastest/cheapest).
    pub fn gemini_flash_lite(&self) -> GeminiModel {
        self.model(GEMINI_FLASH_LITE_LATEST)
    }

    /// Get Gemini 3.1 Pro Preview (latest preview).
    pub fn gemini_31_pro(&self) -> GeminiModel {
        self.model(GEMINI_PRO_PREVIEW_LATEST)
    }

    /// Get Gemini 3 Flash (frontier-class preview).
    pub fn gemini_3_flash(&self) -> GeminiModel {
        self.model(GEMINI_3_FLASH_LATEST)
    }

    /// Get a Gemini model by its model ID.  Any valid Gemini model ID is accepted.
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

        let status = resp.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = match resp.text().await {
                Ok(body) => body,
                Err(e) => {
                    tracing::warn!(status = status_code, error = %e, "Failed to read Gemini error response body");
                    format!("<failed to read response body: {e}>")
                }
            };
            return Err(rusty_ai::AiError::ProviderError {
                provider: "gemini".into(),
                status: Some(status_code),
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
            .map(|m| {
                m.name
                    .strip_prefix("models/")
                    .unwrap_or(&m.name)
                    .to_string()
            })
            .collect())
    }
}
