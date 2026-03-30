//! Anthropic Claude provider for the Rusty AI SDK.
//!
//! This crate implements the `LanguageModel` and `Provider` traits from
//! `rusty_ai` for the Anthropic Messages API. It is **not** a wrapper around
//! an OpenAI-compatible endpoint -- it speaks Anthropic's native API format
//! directly.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use rusty_claude::ClaudeProvider;
//!
//! let provider = ClaudeProvider::new("sk-ant-...");
//! let model = provider.claude_sonnet();
//! ```

mod api_types;
mod convert;
mod model;
mod provider;
mod stream_parser;

pub use model::*;
pub use provider::*;
