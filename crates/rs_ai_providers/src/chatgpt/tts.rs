use async_trait::async_trait;
use rs_ai_core::{AiError, AiResult, AudioResult, TextToSpeechModel, TtsOptions, Usage};

pub struct ChatGptTtsModel {
    api_key: String,
    model_id: String,
    org_id: Option<String>,
}

impl ChatGptTtsModel {
    pub fn new(api_key: String, model_id: String, org_id: Option<String>) -> Self {
        Self {
            api_key,
            model_id,
            org_id,
        }
    }
}

#[async_trait]
impl TextToSpeechModel for ChatGptTtsModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "chatgpt"
    }

    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        options: TtsOptions,
    ) -> AiResult<AudioResult> {
        let client = reqwest::Client::new();

        let mut body = serde_json::json!({
            "model": self.model_id,
            "input": text,
            "voice": voice,
            "response_format": "mp3",
        });

        if let Some(speed) = options.speed {
            body["speed"] = serde_json::json!(speed);
        }

        let mut req = client
            .post("https://api.openai.com/v1/audio/speech")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        if let Some(ref org) = self.org_id {
            req = req.header("OpenAI-Organization", org);
        }

        let resp = req.json(&body).send().await.map_err(|e| {
            AiError::Transport {
                message: format!("OpenAI TTS request failed: {e}"),
                source: Some(Box::new(e)),
            }
        })?;

        let status = resp.status();
        if !status.is_success() {
            let body_text = resp.text().await.unwrap_or_default();
            return Err(AiError::ProviderError {
                provider: "chatgpt".into(),
                status: Some(status.as_u16()),
                message: body_text,
            });
        }

        let audio_bytes = resp.bytes().await.map_err(|e| AiError::Transport {
            message: format!("Failed to read TTS response: {e}"),
            source: Some(Box::new(e)),
        })?;

        Ok(AudioResult {
            audio: audio_bytes.to_vec(),
            mime_type: "audio/mpeg".into(),
            usage: Usage::default(),
        })
    }
}
