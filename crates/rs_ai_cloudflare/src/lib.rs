//!
//! ⚠️ **UNSTABLE** — This crate is in active development. APIs may change without notice.
//!
//! Cloudflare Workers AI provider for RAI.
//!
//! Provides access to Cloudflare's Workers AI models through the unified RAI interface.
//! Cloudflare Workers AI exposes an OpenAI-compatible chat completions API.
//!
//! # Features
//!
//! - **Models**: Llama, Mistral, Gemma, Qwen, and more
//! - **Streaming**: Real-time token streaming via SSE
//! - **Low latency**: Inference runs on Cloudflare's global edge network
//!
//! # Authentication
//!
//! Requires `CF_ACCOUNT_ID` and `CF_API_TOKEN` environment variables, or explicit
//! values passed to [`CloudflareProvider::new`].
//!
//! # Examples
//!
//! ```ignore
//! use rs_ai_cloudflare::CloudflareProvider;
//!
//! let provider = CloudflareProvider::new(
//!     std::env::var("CF_ACCOUNT_ID").unwrap(),
//!     std::env::var("CF_API_TOKEN").unwrap(),
//! );
//! let model = provider.llama_3_1_8b_instruct();
//!
//! let response = model.generate(
//!     rs_ai_traits::Prompt::Text("What is 2+2?".into()),
//!     rs_ai_traits::GenerateOptions::default(),
//! ).await?;
//! ```

mod client;
mod error;
mod model;
mod provider;

pub use error::{CloudflareError, CloudflareResult};
pub use model::CloudflareModel;
pub use provider::CloudflareProvider;

/// Known Cloudflare Workers AI model identifiers.
#[derive(Debug, Clone, Copy)]
pub enum CloudflareModelId {
    /// `@cf/meta/llama-3.1-8b-instruct` — fast, lightweight general-purpose model.
    Llama31_8bInstruct,
    /// `@cf/meta/llama-3.3-70b-instruct-fp8-fast` — high-quality model optimised for speed.
    Llama33_70bInstructFp8Fast,
    /// `@cf/mistral/mistral-7b-instruct-v0.1` — compact instruction-following model.
    Mistral7bInstructV01,
    /// `@cf/google/gemma-7b-it` — Google's Gemma 7B instruction-tuned model.
    Gemma7bIt,
    /// `@cf/qwen/qwen2.5-coder-7b-instruct` — code-focused model from Alibaba.
    Qwen25Coder7bInstruct,
}

impl CloudflareModelId {
    /// Return the model ID string as expected by the Cloudflare Workers AI API.
    pub fn as_str(&self) -> &'static str {
        match self {
            CloudflareModelId::Llama31_8bInstruct => "@cf/meta/llama-3.1-8b-instruct",
            CloudflareModelId::Llama33_70bInstructFp8Fast => {
                "@cf/meta/llama-3.3-70b-instruct-fp8-fast"
            }
            CloudflareModelId::Mistral7bInstructV01 => "@cf/mistral/mistral-7b-instruct-v0.1",
            CloudflareModelId::Gemma7bIt => "@cf/google/gemma-7b-it",
            CloudflareModelId::Qwen25Coder7bInstruct => "@cf/qwen/qwen2.5-coder-7b-instruct",
        }
    }
}
