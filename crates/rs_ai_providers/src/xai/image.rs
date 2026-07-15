use async_trait::async_trait;
use base64::Engine as _;
use rs_ai_core::{
    AiError, AiResult, GeneratedFile, ImageGenerationOptions, ImageModel, ImageResult, Usage,
};

use super::client::{ImageGenerationRequest, XaiClient};

/// xAI image generation model (Grok Imagine / Aurora).
#[derive(Clone)]
pub struct XaiImageModel {
    model_id: String,
    client: std::sync::Arc<XaiClient>,
}

impl XaiImageModel {
    pub fn new(model_id: String, client: std::sync::Arc<XaiClient>) -> Self {
        Self { model_id, client }
    }
}

#[async_trait]
impl ImageModel for XaiImageModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "xai"
    }

    async fn generate_image(
        &self,
        prompt: &str,
        options: ImageGenerationOptions,
    ) -> AiResult<ImageResult> {
        let request = ImageGenerationRequest {
            model: self.model_id.clone(),
            prompt: prompt.to_string(),
            n: options.n,
            size: options.size,
            seed: options.seed,
        };

        let response = self.client.create_image_generation(request).await?;

        if response.data.is_empty() {
            return Err(AiError::ProviderError {
                provider: "xai".to_string(),
                status: None,
                message: "No image data in response".to_string(),
            });
        }

        let mut images: Vec<GeneratedFile> = Vec::new();
        for img in &response.data {
            let result = if let Some(b64) = &img.b64_json {
                let media_type = detect_image_type(b64);
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(b64)
                    .map_err(|e| AiError::ProviderError {
                        provider: "xai".to_string(),
                        status: None,
                        message: format!("Failed to decode base64 image: {e}"),
                    })?;
                Some(GeneratedFile {
                    base64: b64.clone(),
                    bytes,
                    media_type,
                })
            } else if let Some(url) = &img.url {
                let resp = reqwest::get(url).await.map_err(|e| AiError::ProviderError {
                    provider: "xai".to_string(),
                    status: None,
                    message: format!("Failed to fetch image from URL: {e}"),
                })?;
                let raw = resp.bytes().await.map_err(|e| AiError::ProviderError {
                    provider: "xai".to_string(),
                    status: None,
                    message: format!("Failed to read image bytes: {e}"),
                })?;
                let bytes = raw.to_vec();
                let base64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                let media_type = url_media_type(url).unwrap_or("image/png");
                Some(GeneratedFile {
                    base64,
                    bytes,
                    media_type: media_type.to_string(),
                })
            } else {
                None
            };

            if let Some(file) = result {
                images.push(file);
            }
        }

        if images.is_empty() {
            return Err(AiError::ProviderError {
                provider: "xai".to_string(),
                status: None,
                message: "No valid image data in response".to_string(),
            });
        }

        let first = images.remove(0);
        Ok(ImageResult {
            image: first,
            images,
            usage: Usage::default(),
            metadata: Default::default(),
        })
    }
}

/// Detect media type from base64-encoded image data by checking magic bytes.
fn detect_image_type(b64: &str) -> String {
    // Decode first 16 bytes to check magic bytes
    if let Ok(bytes) = base64::engine::general_purpose::STANDARD
        .decode(&b64[..std::cmp::min(24, b64.len())])
    {
        if bytes.len() >= 4 {
            match &bytes[..4] {
                [0x89, b'P', b'N', b'G'] => return "image/png".to_string(),
                [0xFF, 0xD8, _, _] => return "image/jpeg".to_string(),
                [b'R', b'I', b'F', b'F'] => return "image/webp".to_string(),
                [b'G', b'I', b'F', _] => return "image/gif".to_string(),
                _ => {}
            }
        }
    }
    "image/png".to_string()
}

/// Infer media type from URL extension.
fn url_media_type(url: &str) -> Option<&str> {
    let lower = url.to_lowercase();
    if lower.ends_with(".png") {
        Some("image/png")
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        Some("image/jpeg")
    } else if lower.ends_with(".webp") {
        Some("image/webp")
    } else if lower.ends_with(".gif") {
        Some("image/gif")
    } else {
        None
    }
}
