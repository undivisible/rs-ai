use async_trait::async_trait;
use rs_ai_core::{AiError, AiResult, RealtimeEvent, RealtimeSession};

/// Stub xAI realtime session.
///
/// xAI has not yet released a realtime/voice API.
/// This stub exists so the provider compiles with the full trait set.
pub struct XaiRealtimeSession {
    model_id: String,
}

impl XaiRealtimeSession {
    pub fn new(model_id: String) -> Self {
        Self { model_id }
    }
}

#[async_trait]
impl RealtimeSession for XaiRealtimeSession {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "xai"
    }

    async fn send_text(&mut self, _text: &str) -> AiResult<()> {
        Err(AiError::UnsupportedCapability {
            capability: "realtime".to_string(),
            provider: "xai".to_string(),
        })
    }

    async fn send_audio(&mut self, _audio: Vec<u8>, _mime_type: &str) -> AiResult<()> {
        Err(AiError::UnsupportedCapability {
            capability: "realtime".to_string(),
            provider: "xai".to_string(),
        })
    }

    async fn recv(&mut self) -> Option<RealtimeEvent> {
        None
    }

    async fn close(self: Box<Self>) -> AiResult<()> {
        Ok(())
    }
}
