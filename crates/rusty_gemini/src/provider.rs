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

    /// Get the Gemini 2.0 Flash model (general purpose).
    pub fn gemini_pro(&self) -> GeminiModel {
        self.model("gemini-2.0-flash")
    }

    /// Get the Gemini 2.0 Flash Lite model (fast, lightweight).
    pub fn gemini_flash(&self) -> GeminiModel {
        self.model("gemini-2.0-flash-lite")
    }

    /// Get a Gemini model by its model ID.
    pub fn model(&self, model_id: &str) -> GeminiModel {
        use secrecy::ExposeSecret;
        GeminiModel::new(self.api_key.expose_secret(), model_id)
    }
}
