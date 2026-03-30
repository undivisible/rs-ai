/// Detected browser AI capabilities.
#[derive(Debug, Clone)]
pub struct BrowserAiCapabilities {
    /// Whether any browser AI API is available.
    pub available: bool,
    /// Detected browser type.
    pub browser: BrowserType,
    /// Whether the browser AI supports streaming.
    pub supports_streaming: bool,
    /// Whether the browser AI supports system prompts.
    pub supports_system_prompt: bool,
    /// Maximum token limit, if known.
    pub max_tokens: Option<u32>,
}

impl Default for BrowserAiCapabilities {
    fn default() -> Self {
        Self {
            available: false,
            browser: BrowserType::Unknown,
            supports_streaming: false,
            supports_system_prompt: false,
            max_tokens: None,
        }
    }
}

/// Detected browser type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserType {
    Chrome,
    Edge,
    Other(String),
    Unknown,
}

/// Options for browser AI generation.
#[derive(Debug, Clone, Default)]
pub struct BrowserAiOptions {
    pub system_prompt: Option<String>,
    pub temperature: Option<f64>,
    pub top_k: Option<u32>,
}
