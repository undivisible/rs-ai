//! Simplified, convenient API for common use cases.
//!
//! Provides easy-to-use functions for basic AI operations:
//! ```ignore
//! use rai::simple::*;
//!
//! let ai = rai_claude("claude-sonnet-4-6");
//! let result = ai.generate("What is 2+2?").await?;
//! ```

use crate::{AiError, AiResult, GenerateOptions, LanguageModel, Prompt};

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
                Prompt::Text(prompt_str.clone()),
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
/// # Examples
/// ```ignore
/// let ai = rai_claude("claude-sonnet-4-6");
/// let response = ai.generate("Hello, world!").await?;
/// ```
pub async fn rai_claude(_model_id: &str) -> AiResult<SimpleModel> {
    let _api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| AiError::AuthError {
        message: "ANTHROPIC_API_KEY not found. Set the environment variable.".to_string(),
    })?;

    // This will be implemented by the rai_claude provider
    // For now, return a placeholder that will be connected via the provider system
    Err(AiError::ProviderError {
        provider: "claude".to_string(),
        status: None,
        message: "Claude provider integration pending - use rai_claude crate directly".to_string(),
    })
}

/// Factory function for ChatGPT models.
///
/// # Examples
/// ```ignore
/// let ai = rai_chatgpt("gpt-4o");
/// let response = ai.generate("Hello, world!").await?;
/// ```
pub async fn rai_chatgpt(_model_id: &str) -> AiResult<SimpleModel> {
    let _api_key = std::env::var("OPENAI_API_KEY").map_err(|_| AiError::AuthError {
        message: "OPENAI_API_KEY not found. Set the environment variable.".to_string(),
    })?;

    // This will be implemented by the rai_chatgpt provider
    Err(AiError::ProviderError {
        provider: "chatgpt".to_string(),
        status: None,
        message: "ChatGPT provider integration pending - use rai_chatgpt crate directly".to_string(),
    })
}

/// Factory function for Gemini models.
///
/// # Examples
/// ```ignore
/// let ai = rai_gemini("gemini-2.0-flash");
/// let response = ai.generate("Hello, world!").await?;
/// ```
pub async fn rai_gemini(_model_id: &str) -> AiResult<SimpleModel> {
    let _api_key = std::env::var("GOOGLE_API_KEY").map_err(|_| AiError::AuthError {
        message: "GOOGLE_API_KEY not found. Set the environment variable.".to_string(),
    })?;

    // This will be implemented by the rai_gemini provider
    Err(AiError::ProviderError {
        provider: "gemini".to_string(),
        status: None,
        message: "Gemini provider integration pending - use rai_gemini crate directly".to_string(),
    })
}

/// Factory function for OpenAI-compatible endpoints.
///
/// # Examples
/// ```ignore
/// let ai = rai_compatible("https://api.openrouter.ai/api/v1", "openrouter", model_id, api_key).await?;
/// let response = ai.generate("Hello, world!").await?;
/// ```
pub async fn rai_compatible(
    _base_url: &str,
    _preset: Option<&str>,
    _model_id: &str,
    _api_key: &str,
) -> AiResult<SimpleModel> {
    // Will be implemented by rai_openai_compatible provider
    Err(AiError::ProviderError {
        provider: "openai_compatible".to_string(),
        status: None,
        message: "OpenAI compatible provider integration pending".to_string(),
    })
}
