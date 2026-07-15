use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use rs_ai_core::{
    AiError, AiResult, GeneratedFile, ResponseMetadata, Usage, VideoGenerationOptions, VideoModel,
    VideoResult,
};

/// Google Gemini video generation model.
pub struct GeminiVideoModel {
    api_key: String,
    model_id: String,
}

impl GeminiVideoModel {
    pub fn new(api_key: impl Into<String>, model_id: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model_id: model_id.into(),
        }
    }
}

#[async_trait]
impl VideoModel for GeminiVideoModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "gemini"
    }

    async fn generate_video(
        &self,
        prompt: &str,
        options: VideoGenerationOptions,
    ) -> AiResult<VideoResult> {
        let client = reqwest::Client::new();

        let mut params = serde_json::json!({});
        if let Some(n) = options.n {
            params["sampleCount"] = serde_json::json!(n);
        }
        if let Some(r) = &options.resolution {
            params["resolution"] = serde_json::json!(r);
        }
        if let Some(d) = options.duration {
            params["duration"] = serde_json::json!(d);
        }

        let body = serde_json::json!({
            "instances": [{ "prompt": prompt }],
            "parameters": params,
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
                message: format!("Gemini video request failed: {e}"),
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
            message: format!("Gemini video response parse failed: {e}"),
            source: Some(Box::new(e)),
        })?;

        let video_b64 = data["predictions"][0]["bytesBase64Encoded"]
            .as_str()
            .unwrap_or("");
        let file = GeneratedFile {
            base64: video_b64.to_string(),
            bytes: STANDARD.decode(video_b64).unwrap_or_default(),
            media_type: "video/mp4".into(),
        };

        Ok(VideoResult {
            video: file.clone(),
            videos: vec![file],
            usage: Usage::default(),
            metadata: ResponseMetadata::default(),
        })
    }
}
