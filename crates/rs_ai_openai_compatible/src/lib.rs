//! Generic adapter for any OpenAI-compatible chat-completions API.
//!
//! This crate provides [`OpenAiCompatibleProvider`] and [`OpenAiCompatibleModel`]
//! which implement the core `rs_ai_ai` traits and can be pointed at any API that
//! follows the OpenAI chat-completions wire format (OpenAI, Azure, Together,
//! Groq, local vLLM, etc.).

mod api_types;
mod config;
mod convert;
mod model;
mod provider;
mod stream_parser;

pub mod presets;

pub use config::OpenAiCompatibleConfig;
pub use model::OpenAiCompatibleModel;
pub use provider::OpenAiCompatibleProvider;
