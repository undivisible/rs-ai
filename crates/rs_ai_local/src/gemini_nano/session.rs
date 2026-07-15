use std::sync::Arc;

use super::bridge::GeminiNanoBridge;
#[allow(deprecated)]
use super::types::NanoSessionConfig;

/// A multi-turn conversation session backed by Gemini Nano.
///
/// ⚠️ Deprecated — The ML Kit GenAI Prompt API does not support sessions.
///
/// When dropped, the session is automatically closed via a background task.
#[allow(deprecated)]
pub struct NanoSession {
    session_id: String,
    bridge: Arc<dyn GeminiNanoBridge>,
    config: NanoSessionConfig,
}

#[allow(deprecated)]
impl NanoSession {
    pub(crate) fn new(
        session_id: String,
        bridge: Arc<dyn GeminiNanoBridge>,
        config: NanoSessionConfig,
    ) -> Self {
        Self {
            session_id,
            bridge,
            config,
        }
    }

    /// Send a message within this session and receive the model's response.
    ///
    /// ⚠️ Deprecated — Sessions not supported by ML Kit GenAI Prompt API.
    #[allow(deprecated)]
    pub async fn send(&self, message: &str) -> Result<String, rs_ai_core::AiError> {
        self.bridge
            .send_message(&self.session_id, message)
            .await
            .map_err(|e| rs_ai_core::AiError::BridgeError {
                bridge: "gemini_nano".into(),
                message: e,
            })
    }

    /// Returns the identifier for this session.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Returns the configuration used to create this session.
    #[allow(deprecated)]
    pub fn config(&self) -> &NanoSessionConfig {
        &self.config
    }
}

impl Drop for NanoSession {
    #[allow(deprecated)]
    fn drop(&mut self) {
        let bridge = self.bridge.clone();
        let session_id = self.session_id.clone();
        // Fire-and-forget cleanup
        tokio::spawn(async move {
            let _ = bridge.close_session(&session_id).await;
        });
    }
}
