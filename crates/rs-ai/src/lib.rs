//! **rs-ai** - A fluent, ergonomic Rust SDK for AI with 15+ cloud and local providers.
//!
//! # Quick Start
//!
//! ```ignore
//! use rs_ai::gemini;
//!
//! let response = gemini()
//!     .api_key("your-api-key")
//!     .model("gemini-2.5-flash")
//!     .generate("What is 2+2?")
//!     .await?;
//!
//! println!("{}", response);
//! ```
//!
//! # Supported Providers
//!
//! - **Claude** - `rs_ai::claude()`
//! - **ChatGPT** - `rs_ai::chatgpt()`
//! - **Gemini** - `rs_ai::gemini()`
//! - **OpenAI Compatible** - `rs_ai::compatible(base_url)`

use futures::stream::BoxStream;
use rai_ai::{AiError, AiResult, GenerateOptions, LanguageModel, Prompt, StreamEvent};
use rai_chatgpt::ChatGptProvider;
use rai_claude::ClaudeProvider;
use rai_gemini::GeminiProvider;
use rai_openai_compatible::{OpenAiCompatibleConfig, OpenAiCompatibleProvider};

/// Fluent builder for creating and configuring AI clients.
pub struct ClientBuilder {
    provider_type: ProviderType,
    api_key: Option<String>,
    model_id: Option<String>,
}

enum ProviderType {
    Claude,
    ChatGpt,
    Gemini,
    Compatible { base_url: String },
}

impl ClientBuilder {
    /// Set the API key for the provider.
    ///
    /// # Examples
    /// ```ignore
    /// let response = rs_ai::claude()
    ///     .api_key("sk-ant-...")
    ///     .model("claude-sonnet-4-6")
    ///     .generate("Hello!")
    ///     .await?;
    /// ```
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the model ID to use.
    ///
    /// # Examples
    /// ```ignore
    /// let response = rs_ai::gemini()
    ///     .model("gemini-2.5-flash")
    ///     .api_key("...")
    ///     .generate("Hello!")
    ///     .await?;
    /// ```
    pub fn model(mut self, id: impl Into<String>) -> Self {
        self.model_id = Some(id.into());
        self
    }

    /// Generate text from the configured model.
    ///
    /// # Errors
    ///
    /// Returns an error if the API key or model ID is not set, or if the request fails.
    ///
    /// # Examples
    /// ```ignore
    /// let response = rs_ai::claude()
    ///     .api_key("sk-ant-...")
    ///     .model("claude-sonnet-4-6")
    ///     .generate("What is 2+2?")
    ///     .await?;
    /// ```
    pub async fn generate(self, prompt: impl Into<String>) -> AiResult<String> {
        let client = self.build().await?;
        client.generate(prompt).await
    }

    /// Stream text generation from the configured model.
    ///
    /// # Errors
    ///
    /// Returns an error if the API key or model ID is not set, or if the request fails.
    ///
    /// # Examples
    /// ```ignore
    /// use futures::StreamExt;
    ///
    /// let mut stream = rs_ai::claude()
    ///     .api_key("sk-ant-...")
    ///     .model("claude-sonnet-4-6")
    ///     .stream("Hello!")
    ///     .await?;
    ///
    /// while let Some(event) = stream.next().await {
    ///     match event? {
    ///         StreamEvent::TextDelta { text } => print!("{}", text),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub async fn stream(self, prompt: impl Into<String>) -> AiResult<BoxStream<'static, AiResult<StreamEvent>>> {
        let client = self.build().await?;
        client.stream(prompt).await
    }

    async fn build(self) -> AiResult<Client> {
        let model_id = self.model_id.ok_or_else(|| AiError::ProviderError {
            provider: "rs-ai".to_string(),
            status: None,
            message: "Model ID not set. Use .model() to specify a model.".to_string(),
        })?;

        let api_key = self.api_key.ok_or_else(|| AiError::AuthError {
            message: "API key not set. Use .api_key() to specify credentials.".to_string(),
        })?;

        let model: Box<dyn LanguageModel> = match self.provider_type {
            ProviderType::Claude => {
                let provider = ClaudeProvider::new(api_key);
                Box::new(provider.model(&model_id))
            }
            ProviderType::ChatGpt => {
                let provider = ChatGptProvider::new(api_key);
                Box::new(provider.model(&model_id))
            }
            ProviderType::Gemini => {
                let provider = GeminiProvider::new(api_key);
                Box::new(provider.model(&model_id))
            }
            ProviderType::Compatible { base_url } => {
                let config = OpenAiCompatibleConfig::new(&base_url, &api_key);
                let provider = OpenAiCompatibleProvider::new(config, "custom", "OpenAI Compatible");
                provider.language_model(&model_id)
            }
        };

        Ok(Client { model })
    }
}

/// A configured AI client ready for operations.
pub struct Client {
    model: Box<dyn LanguageModel>,
}

impl Client {
    /// Generate text from the model.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn generate(&self, prompt: impl Into<String>) -> AiResult<String> {
        let prompt_str = prompt.into();
        let result = self
            .model
            .generate(Prompt::Text(prompt_str), GenerateOptions::default())
            .await?;

        result.text.ok_or_else(|| AiError::ProviderError {
            provider: self.model.provider_id().to_string(),
            status: None,
            message: "No text in response (model returned only tool calls)".to_string(),
        })
    }

    /// Stream text generation from the model.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn stream(&self, prompt: impl Into<String>) -> AiResult<BoxStream<'static, AiResult<StreamEvent>>> {
        let prompt_str = prompt.into();
        self.model
            .stream(Prompt::Text(prompt_str), GenerateOptions::default())
            .await
    }

    /// Get a reference to the underlying language model.
    pub fn model(&self) -> &dyn LanguageModel {
        &*self.model
    }
}

/// Create a client builder for Claude.
///
/// # Examples
/// ```ignore
/// let response = rs_ai::claude()
///     .api_key("sk-ant-...")
///     .model("claude-sonnet-4-6")
///     .generate("Hello!")
///     .await?;
/// ```
pub fn claude() -> ClientBuilder {
    ClientBuilder {
        provider_type: ProviderType::Claude,
        api_key: None,
        model_id: None,
    }
}

/// Create a client builder for ChatGPT.
///
/// # Examples
/// ```ignore
/// let response = rs_ai::chatgpt()
///     .api_key("sk-...")
///     .model("gpt-4o")
///     .generate("Hello!")
///     .await?;
/// ```
pub fn chatgpt() -> ClientBuilder {
    ClientBuilder {
        provider_type: ProviderType::ChatGpt,
        api_key: None,
        model_id: None,
    }
}

/// Create a client builder for Gemini.
///
/// # Examples
/// ```ignore
/// let response = rs_ai::gemini()
///     .api_key("AIzaSy...")
///     .model("gemini-2.5-flash")
///     .generate("Hello!")
///     .await?;
/// ```
pub fn gemini() -> ClientBuilder {
    ClientBuilder {
        provider_type: ProviderType::Gemini,
        api_key: None,
        model_id: None,
    }
}

/// Create a client builder for an OpenAI-compatible endpoint.
///
/// # Examples
/// ```ignore
/// let response = rs_ai::compatible("https://api.openrouter.ai/api/v1")
///     .api_key("sk-...")
///     .model("meta-llama/llama-2-70b-chat")
///     .generate("Hello!")
///     .await?;
/// ```
pub fn compatible(base_url: impl Into<String>) -> ClientBuilder {
    ClientBuilder {
        provider_type: ProviderType::Compatible {
            base_url: base_url.into(),
        },
        api_key: None,
        model_id: None,
    }
}
