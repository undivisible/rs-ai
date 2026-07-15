//!
//! ⚠️ **UNSTABLE** — This crate is in active development. APIs may change without notice.
//!
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
//! use rs_ai_providers::xai::XaiProvider;
//!
//! let provider = XaiProvider::new("your-api-key");
//! let model = provider.grok_4_20_reasoning();
//!
//! let response = model.generate(
//!     rs_ai_core::Prompt::Text("What is 2+2?".into()),
//!     rs_ai_core::GenerateOptions::default(),
//! ).await?;
//! ```

mod client;
mod error;
mod image;
mod model;
mod provider;
pub mod realtime;
pub mod speech;
pub mod transcription;
pub mod video;

pub use client::{AURORA, GROK_4_IMAGINE};
pub use error::{XaiError, XaiResult};
pub use image::XaiImageModel;
pub use model::XaiModel;
pub use provider::XaiProvider;
pub use speech::XaiSpeechModel;
pub use transcription::XaiTranscriptionModel;
pub use video::XaiVideoModel;

pub const XAI_OAUTH_MODEL_IDS: &[&str] = &[
    "grok-4.5",
    "grok-4.3",
    "grok-build",
    "grok-composer-2.5-fast",
    "grok-4.20-0309-reasoning",
    "grok-4.20-0309-non-reasoning",
    "grok-4.20-multi-agent-0309",
];

#[derive(Debug, Clone, Copy)]
pub enum XaiModelId {
    Grok420Reasoning,
    Grok4,
    Grok4Imagine,
    Aurora,
}

impl XaiModelId {
    pub fn as_str(&self) -> &'static str {
        match self {
            XaiModelId::Grok420Reasoning => "grok-4.20-reasoning",
            XaiModelId::Grok4 => "grok-4",
            XaiModelId::Grok4Imagine => GROK_4_IMAGINE,
            XaiModelId::Aurora => AURORA,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::XAI_OAUTH_MODEL_IDS;

    #[test]
    fn oauth_catalog_includes_grok_4_5() {
        assert!(XAI_OAUTH_MODEL_IDS.contains(&"grok-4.5"));
    }

    #[test]
    fn oauth_catalog_includes_grok_build() {
        assert!(XAI_OAUTH_MODEL_IDS.contains(&"grok-build"));
    }
}
