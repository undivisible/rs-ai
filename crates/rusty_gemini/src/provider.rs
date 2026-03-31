use secrecy::SecretString;

use crate::model::GeminiModel;

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

    /// Get Gemini 2.5 Pro (most capable).
    pub fn gemini_pro(&self) -> GeminiModel {
        self.model("gemini-2.5-pro")
    }

    /// Get Gemini 2.5 Flash (best price/performance).
    pub fn gemini_flash(&self) -> GeminiModel {
        self.model("gemini-2.5-flash")
    }

    /// Get Gemini 2.5 Flash Lite (fastest/cheapest).
    pub fn gemini_flash_lite(&self) -> GeminiModel {
        self.model("gemini-2.5-flash-lite")
    }

    /// Get a Gemini model by its model ID.
    pub fn model(&self, model_id: &str) -> GeminiModel {
        use secrecy::ExposeSecret;
        GeminiModel::new(self.api_key.expose_secret(), model_id)
    }
}
