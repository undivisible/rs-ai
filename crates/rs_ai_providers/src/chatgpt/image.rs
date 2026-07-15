//!
//! ⚠️ **UNSTABLE** — This crate is in active development. APIs may change without notice.
//!
//! DALL-E image generation via the OpenAI Images API.
//!
//! # Example
//!
//! ```rust,no_run
//! use rs_ai_providers::chatgpt::ChatGptImageModel;
//! use rs_ai_core::{ImageModel, ImageGenerationOptions};
//!
//! # async fn example() -> rs_ai_core::AiResult<()> {
//! let model = ChatGptImageModel::new("dall-e-3", "sk-...", "https://api.openai.com/v1");
//! let result = model.generate_image("A cat wearing a hat", ImageGenerationOptions::default()).await?;
//! println!("got {} images", result.images.len());
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use rs_ai_core::{
    AiError, AiResult, GeneratedFile, ImageGenerationOptions, ImageModel, ImageResult,
    ResponseMetadata, Usage,
};

/// Request body for POST /v1/images/generations.
#[derive(serde::Serialize)]
struct ImagesRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<String>,
    /// Always "b64_json" to avoid extra URL fetch per image.
    response_format: String,
}

#[derive(serde::Deserialize)]
struct ImagesResponse {
    #[allow(dead_code)]
    created: i64,
    data: Vec<ImageData>,
}

#[derive(serde::Deserialize)]
struct ImageData {
    b64_json: String,
    #[allow(dead_code)]
    revised_prompt: Option<String>,
}

/// DALL-E image generation model.
pub struct ChatGptImageModel {
    model_id: String,
    api_key: String,
    base_url: String,
    org_id: Option<String>,
}

impl ChatGptImageModel {
    /// Create a new DALL-E model.
    ///
    /// `base_url` should be the OpenAI API root (e.g. `https://api.openai.com/v1`).
    pub fn new(
        model_id: impl Into<String>,
        api_key: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            model_id: model_id.into(),
            api_key: api_key.into(),
            base_url: base_url.into(),
            org_id: None,
        }
    }

    /// Set an optional OpenAI organization ID.
    pub fn with_org(mut self, org_id: impl Into<String>) -> Self {
        self.org_id = Some(org_id.into());
        self
    }

    fn is_dalle3(&self) -> bool {
        self.model_id.contains("dall-e-3")
    }
}

#[async_trait]
impl ImageModel for ChatGptImageModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "chatgpt"
    }

    async fn generate_image(
        &self,
        prompt: &str,
        options: ImageGenerationOptions,
    ) -> AiResult<ImageResult> {
        // DALL-E 3 only supports n=1.
        if self.is_dalle3() {
            if let Some(n) = options.n {
                if n != 1 {
                    return Err(AiError::ProviderError {
                        provider: "chatgpt".into(),
                        status: None,
                        message: "DALL-E 3 only supports n=1".into(),
                    });
                }
            }
        }

        // Forward quality/style from provider_options (DALL-E 3 only).
        let quality = if self.is_dalle3() {
            options
                .provider_options
                .as_ref()
                .and_then(|o| o.get("quality").and_then(|v| v.as_str()))
                .map(|s| s.to_string())
        } else {
            None
        };
        let style = if self.is_dalle3() {
            options
                .provider_options
                .as_ref()
                .and_then(|o| o.get("style").and_then(|v| v.as_str()))
                .map(|s| s.to_string())
        } else {
            None
        };

        let request = ImagesRequest {
            model: self.model_id.clone(),
            prompt: prompt.to_string(),
            n: options.n.or(Some(1)),
            size: options.size.clone(),
            quality,
            style,
            response_format: "b64_json".to_string(),
        };

        let client = reqwest::Client::new();
        let mut req = client
            .post(format!("{}/images/generations", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request);

        if let Some(ref org) = self.org_id {
            req = req.header("OpenAI-Organization", org);
        }

        let resp = req.send().await.map_err(|e| AiError::Transport {
            message: e.to_string(),
            source: Some(Box::new(e)),
        })?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AiError::ProviderError {
                provider: "chatgpt".into(),
                status: Some(status.as_u16()),
                message: body,
            });
        }

        let images_resp: ImagesResponse =
            resp.json().await.map_err(|e| AiError::Serialization(e.to_string()))?;

        let mut images = Vec::with_capacity(images_resp.data.len());
        for img in images_resp.data {
            let bytes =
                STANDARD.decode(&img.b64_json).map_err(|e| AiError::Serialization(e.to_string()))?;
            images.push(GeneratedFile {
                base64: img.b64_json,
                bytes,
                media_type: "image/png".to_string(),
            });
        }

        let image = images.first().cloned().ok_or_else(|| AiError::ProviderError {
            provider: "chatgpt".into(),
            status: None,
            message: "No images returned by API".into(),
        })?;

        Ok(ImageResult {
            image,
            images,
            usage: Usage::default(),
            metadata: ResponseMetadata::new("chatgpt", &self.model_id),
        })
    }
}
