//!
//! ⚠️ **UNSTABLE** — This crate is in active development. APIs may change without notice.
//!
//! Cloud AI providers for the Rusty AI SDK.
//!
//! Each provider is gated behind a Cargo feature flag:
//! - `claude` — Anthropic Claude
//! - `chatgpt` — OpenAI ChatGPT
//! - `gemini` — Google Gemini
//! - `openai-compatible` — Generic OpenAI-compatible adapter
//! - `xai` — xAI Grok
//! - `cloudflare` — Cloudflare Workers AI
//! - `ollama` — Local Ollama
//! - `portkey` — Portkey AI Gateway
//! - `langfuse` — Langfuse observability wrapper

#[cfg(feature = "chatgpt")]
pub mod chatgpt;
#[cfg(feature = "claude")]
pub mod claude;
#[cfg(feature = "cloudflare")]
pub mod cloudflare;
#[cfg(feature = "gemini")]
pub mod gemini;
#[cfg(feature = "langfuse")]
pub mod langfuse;
#[cfg(feature = "ollama")]
pub mod ollama;
#[cfg(feature = "openai-compatible")]
pub mod openai_compatible;
#[cfg(feature = "portkey")]
pub mod portkey;
#[cfg(feature = "xai")]
pub mod xai;

// Re-exports for convenience when the corresponding feature is enabled.
#[cfg(feature = "chatgpt")]
pub use chatgpt::*;
#[cfg(feature = "claude")]
pub use claude::*;
#[cfg(feature = "cloudflare")]
pub use cloudflare::*;
#[cfg(feature = "gemini")]
pub use gemini::*;
#[cfg(feature = "langfuse")]
pub use langfuse::*;
#[cfg(feature = "ollama")]
pub use ollama::*;
#[cfg(feature = "openai-compatible")]
pub use openai_compatible::*;
#[cfg(feature = "portkey")]
pub use portkey::*;
#[cfg(feature = "xai")]
pub use xai::*;
