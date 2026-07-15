use async_trait::async_trait;

use rs_ai_core::{
    AiError, AiResult, VideoGenerationOptions, VideoModel, VideoResult,
};

/// Model constant for Google Veo.
pub const VEO_3: &str = "veo-3.0-generate-001";

/// A Gemini Veo video generation model.
///
/// ⚠️ Veo video generation is not yet available through this SDK.
/// This implementation is a stub that returns an `AiError::Unsupported` error.
pub struct GeminiVideoModel;

impl GeminiVideoModel {
    /// Create a new Gemini Veo model.
    pub fn new() -> Self {
        Self
    }
}

impl Default for GeminiVideoModel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VideoModel for GeminiVideoModel {
    fn model_id(&self) -> &str {
        "veo-3.0-generate-001"
    }

    fn provider_id(&self) -> &str {
        "gemini"
    }

    async fn generate_video(
        &self,
        _prompt: &str,
        _options: VideoGenerationOptions,
    ) -> AiResult<VideoResult> {
        Err(AiError::UnsupportedCapability {
            provider: "gemini".to_string(),
            capability: "Veo video generation".to_string(),
        })
    }
}
