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

    /// Get a Gemini model with a custom base URL (e.g. for Cloudflare AI Gateway).
    pub fn model_with_base_url(&self, model_id: &str, base_url: impl Into<String>) -> GeminiModel {
        use secrecy::ExposeSecret;
        GeminiModel::new(self.api_key.expose_secret(), model_id).with_base_url(base_url)
    }

    /// Open a Gemini Live API session for bidirectional voice/video streaming.
    ///
    /// # Example
    /// ```no_run
    /// use rai_gemini::{GeminiProvider, live_api::{LiveEvent, LiveGenerationConfig, BidiSetup}};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = GeminiProvider::new(std::env::var("GOOGLE_API_KEY")?);
    /// let mut session = provider
    ///     .live_session("gemini-2.5-flash-live-preview")
    ///     .await?;
    ///
    /// session.send_text("Hello, can you hear me?").await?;
    /// while let Some(ev) = session.recv().await {
    ///     match ev? {
    ///         LiveEvent::TextDelta(t) => print!("{t}"),
    ///         LiveEvent::TurnComplete => break,
    ///         _ => {}
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn live_session(
        &self,
        model_id: &str,
    ) -> Result<crate::live_api::LiveSession, crate::live_api::LiveError> {
        use crate::live_api::{BidiSetup, LiveGenerationConfig};
        use secrecy::ExposeSecret;
        let setup = BidiSetup {
            model: format!("models/{model_id}"),
            system_instruction: None,
            generation_config: Some(LiveGenerationConfig {
                response_modalities: Some(vec!["AUDIO".into(), "TEXT".into()]),
                ..Default::default()
            }),
            tools: None,
        };
        crate::live_api::LiveSession::connect(self.api_key.expose_secret(), model_id, setup).await
    }

    /// Fetch the list of models from the Gemini API.
    ///
    /// Calls `GET /v1beta/models?key=...` and returns model names.
    pub async fn list_remote_models(&self) -> rai_ai::AiResult<Vec<String>> {
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
            .map_err(|e| rai_ai::AiError::Transport {
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
            return Err(rai_ai::AiError::ProviderError {
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
            .map_err(|e| rai_ai::AiError::Serialization(e.to_string()))?;

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
