//!
//! ⚠️ **UNSTABLE** — This crate is in active development. APIs may change without notice.
//!
//! Portkey AI Gateway provider for RAI.
//!
//! Provides access to Portkey's multi-provider LLM gateway through the unified RAI interface.
//! Portkey acts as a proxy/gateway layer, routing requests to any provider with support for:
//! - Multi-provider routing and load balancing
//! - Automatic fallback and retry logic
//! - Response caching and request tracking
//! - Analytics and observability
//!
//! # Features
//!
//! - **Multi-provider routing**: Route requests to any LLM provider
//! - **Load balancing**: Automatic distribution across providers
//! - **Fallback**: Automatic retry with fallback providers
//! - **Caching**: Response caching for common requests
//! - **Observability**: Built-in analytics and tracking
//! - **Text generation**: Full support for text-based prompts and responses
//! - **Streaming**: Real-time streaming responses
//!
//! # Authentication
//!
//! Requires `PORTKEY_API_KEY` environment variable, or explicit value passed to [`PortkeyProvider::new`].
//!
//! # Examples
//!
//! ```ignore
//! use rs_ai_providers::portkey::PortkeyProvider;
//!
//! let provider = PortkeyProvider::new(
//!     std::env::var("PORTKEY_API_KEY").unwrap(),
//! );
//! let model = provider.model("gpt-4");
//!
//! let response = model.generate(
//!     rs_ai_core::Prompt::Text("What is 2+2?".into()),
//!     rs_ai_core::GenerateOptions::default(),
//! ).await?;
//! ```

mod client;
mod error;
mod model;
mod provider;

pub use error::{PortkeyError, PortkeyResult};
pub use model::PortkeyModel;
pub use provider::PortkeyProvider;
