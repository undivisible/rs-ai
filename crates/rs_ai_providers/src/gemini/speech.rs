use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use rs_ai_core::{AiError, AiResult, AudioResult, TextToSpeechModel, TtsOptions, Usage};

/// Google Gemini text-to-speech model.
pub struct GeminiSpeechModel {
    api_key: String,
    model_id: String,
}

impl GeminiSpeechModel {
    pub fn new(api_key: impl Into<String>, model_id: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model_id: model_id.into(),
        }
    }
}

#[async_trait]
impl TextToSpeechModel for GeminiSpeechModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "gemini"
    }

    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        _options: TtsOptions,
    ) -> AiResult<AudioResult> {
        let client = reqwest::Client::new();

        let body = serde_json::json!({
            "instances": [{
                "text": text,
            }],
            "parameters": {
                "voice": voice,
            }
        });

        let url = format!(
            "https://generativelanguage.googleapis.com/v1/models/{}:predict?key={}",
            self.model_id, self.api_key
        );

        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::Transport {
                message: format!("Gemini TTS request failed: {e}"),
                source: Some(Box::new(e)),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(AiError::ProviderError {
                provider: "gemini".into(),
                status: Some(status_code),
                message: body_text,
            });
        }

        let data: serde_json::Value = resp.json().await.map_err(|e| AiError::Transport {
            message: format!("Gemini TTS response parse failed: {e}"),
            source: Some(Box::new(e)),
        })?;

        let audio_b64 = data["predictions"][0]["bytesBase64Encoded"]
            .as_str()
            .unwrap_or("");
        let audio = STANDARD.decode(audio_b64).unwrap_or_default();

        Ok(AudioResult {
            audio,
            mime_type: "audio/pcm".into(),
            usage: Usage::default(),
        })
    }
}
