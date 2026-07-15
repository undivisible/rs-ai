use async_trait::async_trait;
use reqwest::multipart;
use rs_ai_core::{AiError, AiResult, SpeechToTextModel, TranscriptionResult, Usage};

#[derive(serde::Deserialize)]
struct XaiTranscriptionResponse {
    text: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
}

/// xAI speech-to-text model (Grok Voice).
pub struct XaiTranscriptionModel {
    api_key: String,
    model_id: String,
}

impl XaiTranscriptionModel {
    pub fn new(api_key: String, model_id: String) -> Self {
        Self { api_key, model_id }
    }
}

#[async_trait]
impl SpeechToTextModel for XaiTranscriptionModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "xai"
    }

    async fn transcribe(
        &self,
        audio: Vec<u8>,
        mime_type: &str,
        language: Option<&str>,
    ) -> AiResult<TranscriptionResult> {
        let client = reqwest::Client::new();

        let ext = mime_type.rsplit('/').next().unwrap_or("webm");
        let part = multipart::Part::bytes(audio)
            .mime_str(mime_type)
            .map_err(|e| AiError::Transport {
                message: format!("MIME type error: {e}"),
                source: Some(Box::new(e)),
            })?
            .file_name(format!("audio.{ext}"));

        let mut form = multipart::Form::new()
            .part("file", part)
            .text("model", self.model_id.clone())
            .text("response_format", "json");

        if let Some(lang) = language {
            form = form.text("language", lang.to_string());
        }

        let resp = client
            .post("https://api.x.ai/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| AiError::Transport {
                message: format!("xAI STT request failed: {e}"),
                source: Some(Box::new(e)),
            })?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AiError::ProviderError {
                provider: "xai".into(),
                status: Some(status.as_u16()),
                message: body,
            });
        }

        let data: XaiTranscriptionResponse = resp.json().await.map_err(|e| AiError::Transport {
            message: format!("Failed to parse xAI STT response: {e}"),
            source: Some(Box::new(e)),
        })?;

        Ok(TranscriptionResult {
            text: data.text,
            language: data.language,
            duration_seconds: data.duration,
            usage: Usage::default(),
        })
    }
}
