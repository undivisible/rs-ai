use async_trait::async_trait;
use rs_ai_core::types::ResponseMetadata;
use rs_ai_core::Usage;
use rs_ai_core::{
    AiError, AiResult, GeneratedFile, VideoGenerationOptions, VideoModel, VideoResult,
};

/// xAI video generation model (async poll-based API).
pub struct XaiVideoModel {
    api_key: String,
    model_id: String,
}

impl XaiVideoModel {
    pub fn new(api_key: String, model_id: String) -> Self {
        Self { api_key, model_id }
    }
}

#[async_trait]
impl VideoModel for XaiVideoModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "xai"
    }

    async fn generate_video(
        &self,
        prompt: &str,
        options: VideoGenerationOptions,
    ) -> AiResult<VideoResult> {
        let client = reqwest::Client::new();

        let mut body = serde_json::json!({
            "model": self.model_id,
            "prompt": prompt,
        });
        if let Some(n) = options.n {
            body["n"] = serde_json::json!(n);
        }
        if let Some(r) = &options.resolution {
            body["resolution"] = serde_json::json!(r);
        }
        if let Some(d) = options.duration {
            body["duration"] = serde_json::json!(d);
        }

        // Create generation
        let create_resp = client
            .post("https://api.x.ai/v1/video/generations")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::Transport {
                message: format!("xAI video create failed: {e}"),
                source: Some(Box::new(e)),
            })?;

        let status = create_resp.status();
        if !status.is_success() {
            let body_text = create_resp.text().await.unwrap_or_default();
            return Err(AiError::ProviderError {
                provider: "xai".into(),
                status: Some(status.as_u16()),
                message: body_text,
            });
        }

        let create_data: serde_json::Value =
            create_resp.json().await.map_err(|e| AiError::Transport {
                message: format!("xAI video parse failed: {e}"),
                source: Some(Box::new(e)),
            })?;

        let generation_id = create_data["id"]
            .as_str()
            .ok_or_else(|| AiError::ProviderError {
                provider: "xai".into(),
                status: None,
                message: "No generation ID in response".into(),
            })?
            .to_string();

        // Poll until complete
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            let poll_resp = client
                .get(format!(
                    "https://api.x.ai/v1/video/generations/{}",
                    generation_id
                ))
                .header("Authorization", format!("Bearer {}", self.api_key))
                .send()
                .await
                .map_err(|e| AiError::Transport {
                    message: format!("xAI video poll failed: {e}"),
                    source: Some(Box::new(e)),
                })?;

            let poll_status = poll_resp.status();
            if !poll_status.is_success() {
                let body_text = poll_resp.text().await.unwrap_or_default();
                return Err(AiError::ProviderError {
                    provider: "xai".into(),
                    status: Some(poll_status.as_u16()),
                    message: body_text,
                });
            }

            let poll_data: serde_json::Value =
                poll_resp.json().await.map_err(|e| AiError::Transport {
                    message: format!("xAI video poll parse failed: {e}"),
                    source: Some(Box::new(e)),
                })?;

            let status = poll_data["status"].as_str().unwrap_or("");
            match status {
                "completed" | "succeeded" => {
                    let video_url = poll_data["output"]["video_url"]
                        .as_str()
                        .or_else(|| poll_data["output"][0]["url"].as_str())
                        .unwrap_or("");

                    let file = GeneratedFile {
                        base64: String::new(),
                        bytes: video_url.as_bytes().to_vec(),
                        media_type: "video/mp4".into(),
                    };

                    return Ok(VideoResult {
                        video: file.clone(),
                        videos: vec![file],
                        usage: Usage::default(),
                        metadata: ResponseMetadata::default(),
                    });
                }
                "failed" => {
                    let err = poll_data["error"].as_str().unwrap_or("unknown error");
                    return Err(AiError::ProviderError {
                        provider: "xai".into(),
                        status: None,
                        message: err.into(),
                    });
                }
                _ => {
                    // still processing, continue polling
                }
            }
        }
    }
}
