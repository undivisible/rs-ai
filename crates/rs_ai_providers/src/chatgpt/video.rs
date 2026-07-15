//!
//! ⚠️ **UNSTABLE** — This crate is in active development. APIs may change without notice.
//!
//! Sora video generation for OpenAI ChatGPT provider.
//!
//! The Sora API is not yet publicly available.  This module provides a
//! `ChatGptVideoModel` that implements the `VideoModel` trait but returns
//! [`AiError::ProviderError`] with a message indicating the API is not yet available.
//!
//! # Example
//!
//! ```rust,no_run
//! use rs_ai_providers::chatgpt::ChatGptVideoModel;
//! use rs_ai_core::{VideoModel, VideoGenerationOptions};
//!
//! # async fn example() -> rs_ai_core::AiResult<()> {
//! let model = ChatGptVideoModel::new("sora-2");
//! let result = model.generate_video("A cat wearing a hat", VideoGenerationOptions::default()).await;
//! assert!(result.is_err());
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use rs_ai_core::{AiError, AiResult, VideoGenerationOptions, VideoModel, VideoResult};

/// Sora video generation model (stub).
///
/// Returns `AiError::ProviderError` until the OpenAI Sora API becomes available.
pub struct ChatGptVideoModel {
    model_id: String,
}

impl ChatGptVideoModel {
    /// Create a new Sora video model stub.
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
        }
    }
}

#[async_trait]
impl VideoModel for ChatGptVideoModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "chatgpt"
    }

    async fn generate_video(
        &self,
        _prompt: &str,
        _options: VideoGenerationOptions,
    ) -> AiResult<VideoResult> {
        Err(AiError::ProviderError {
            provider: "chatgpt".into(),
            status: None,
            message: "Sora video generation is not yet available via the OpenAI API".into(),
        })
    }
}
