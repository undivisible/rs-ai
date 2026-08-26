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
//!
//! The `catalog` module (always on) provides static metadata for
//! API-key-based providers — id, name, env vars, base URLs, models.

pub mod catalog;

#[cfg(feature = "chatgpt")]
pub mod chatgpt;
#[cfg(feature = "claude")]
pub mod claude;
#[cfg(feature = "cloudflare")]
pub mod cloudflare;
#[cfg(feature = "cohere")]
pub mod cohere;
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
#[cfg(feature = "voyage")]
pub mod voyage;
#[cfg(feature = "xai")]
pub mod xai;

// Re-exports for convenience when the corresponding feature is enabled.
#[cfg(feature = "chatgpt")]
pub use chatgpt::*;
#[cfg(feature = "claude")]
pub use claude::*;
#[cfg(feature = "cloudflare")]
pub use cloudflare::*;
#[cfg(feature = "cohere")]
pub use cohere::{CohereProvider, CohereRerankingModel, RERANK_MODEL as COHERE_RERANK_MODEL};
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
#[cfg(feature = "voyage")]
pub use voyage::{VoyageRerankingModel, RERANK_LITE, RERANK_MODEL as VOYAGE_RERANK_MODEL};
#[cfg(feature = "xai")]
pub use xai::*;
