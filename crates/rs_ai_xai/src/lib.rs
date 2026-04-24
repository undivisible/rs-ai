//! xAI Grok provider for RAI.
//!
//! Provides access to xAI's Grok models through the unified RAI interface.
//!
//! # Features
//!
//! - **Models**: Grok-4.20-Reasoning, Grok-4, and more
//! - **Vision**: Image understanding and analysis
//! - **Streaming**: Real-time token streaming
//! - **Tool Use**: Function calling and tool integration
//!
//! # Authentication
//!
//! Requires the `XAI_API_KEY` environment variable or explicit API key configuration.
//!
//! # Examples
//!
//! ```ignore
//! use rs_ai_xai::XaiProvider;
//!
//! let provider = XaiProvider::new("your-api-key");
//! let model = provider.grok_4_20_reasoning();
//!
//! let response = model.generate(
//!     rs_ai_ai::Prompt::Text("What is 2+2?".into()),
//!     rs_ai_ai::GenerateOptions::default(),
//! ).await?;
//! ```

mod client;
mod error;
mod model;
mod provider;

pub use error::{XaiError, XaiResult};
pub use model::XaiModel;
pub use provider::XaiProvider;

#[derive(Debug, Clone, Copy)]
pub enum XaiModelId {
    Grok420Reasoning,
    Grok4,
}

impl XaiModelId {
    pub fn as_str(&self) -> &'static str {
        match self {
            XaiModelId::Grok420Reasoning => "grok-4.20-reasoning",
            XaiModelId::Grok4 => "grok-4",
        }
    }
}
