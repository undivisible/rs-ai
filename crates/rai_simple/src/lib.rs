//! Simplified, convenient API for common RAI use cases.
//!
//! Provides easy-to-use factory functions for quick AI operations:
//! ```ignore
//! use rai_simple::*;
//!
//! let ai = rai_claude("claude-sonnet-4-6")?;
//! let result = ai.generate("What is 2+2?").await?;
//! ```

use rai_ai::{AiError, AiResult, GenerateOptions, LanguageModel, Prompt};
use rai_claude::ClaudeProvider;
use rai_chatgpt::ChatGptProvider;
use rai_gemini::GeminiProvider;
use rai_openai_compatible::{OpenAiCompatibleConfig, OpenAiCompatibleProvider};

/// A simplified wrapper around a language model for easy use.
pub struct SimpleModel {
    model: Box<dyn LanguageModel>,
}

impl SimpleModel {
    /// Create a new simple model wrapper.
    pub fn new(model: Box<dyn LanguageModel>) -> Self {
        Self { model }
    }

    /// Generate text from the model.
    pub async fn generate(&self, prompt: impl Into<String>) -> AiResult<String> {
        let prompt_str = prompt.into();
        let result = self
            .model
            .generate(
                Prompt::Text(prompt_str),
                GenerateOptions::default(),
            )
            .await?;

        result.text.ok_or_else(|| AiError::ProviderError {
            provider: self.model.provider_id().to_string(),
            status: None,
            message: "No text in response (model returned only tool calls)".to_string(),
        })
    }

    /// Get a reference to the underlying model.
    pub fn model(&self) -> &dyn LanguageModel {
        &*self.model
    }
}

/// Factory function for Claude models.
///
/// Requires `ANTHROPIC_API_KEY` environment variable.
///
/// # Examples
/// ```ignore
/// let ai = rai_claude("claude-sonnet-4-6")?;
/// let response = ai.generate("Hello, world!").await?;
/// ```
pub fn rai_claude(model_id: &str) -> AiResult<SimpleModel> {
    let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| AiError::AuthError {
        message: "ANTHROPIC_API_KEY not found. Set the environment variable.".to_string(),
    })?;

    let provider = ClaudeProvider::new(api_key);
    let model = provider.model(model_id);
    Ok(SimpleModel::new(Box::new(model)))
}

/// Factory function for ChatGPT models.
///
/// Requires `OPENAI_API_KEY` environment variable.
///
/// # Examples
/// ```ignore
/// let ai = rai_chatgpt("gpt-4o")?;
/// let response = ai.generate("Hello, world!").await?;
/// ```
pub fn rai_chatgpt(model_id: &str) -> AiResult<SimpleModel> {
    let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| AiError::AuthError {
        message: "OPENAI_API_KEY not found. Set the environment variable.".to_string(),
    })?;

    let provider = ChatGptProvider::new(api_key);
    let model = provider.model(model_id);
    Ok(SimpleModel::new(Box::new(model)))
}

/// Factory function for Gemini models.
///
/// Requires `GOOGLE_API_KEY` environment variable.
///
/// # Examples
/// ```ignore
/// let ai = rai_gemini("gemini-2.5-flash")?;
/// let response = ai.generate("Hello, world!").await?;
/// ```
pub fn rai_gemini(model_id: &str) -> AiResult<SimpleModel> {
    let api_key = std::env::var("GOOGLE_API_KEY").map_err(|_| AiError::AuthError {
        message: "GOOGLE_API_KEY not found. Set the environment variable.".to_string(),
    })?;

    let provider = GeminiProvider::new(api_key);
    let model = provider.model(model_id);
    Ok(SimpleModel::new(Box::new(model)))
}

/// Factory function for OpenAI-compatible endpoints.
///
/// # Examples
/// ```ignore
/// let ai = rai_compatible(
///     "https://api.openrouter.ai/api/v1",
///     None,
///     "meta-llama/llama-2-7b",
///     api_key,
/// )?;
/// let response = ai.generate("Hello, world!").await?;
/// ```
pub fn rai_compatible(
    base_url: &str,
    _preset: Option<&str>,
    model_id: &str,
    api_key: &str,
) -> AiResult<SimpleModel> {
    let config = OpenAiCompatibleConfig::new(base_url, api_key);
    let provider = OpenAiCompatibleProvider::new(config, "custom", "Custom OpenAI-Compatible");
    let model = provider.language_model(model_id);
    Ok(SimpleModel::new(model))
}
