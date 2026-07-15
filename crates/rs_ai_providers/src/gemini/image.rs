use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use serde::Deserialize;

use rs_ai_core::{
    AiError, AiResult, GeneratedFile, ImageGenerationOptions, ImageModel, ImageResult,
    ResponseMetadata, Usage,
};

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1";

/// Model constants for Google Imagen.
pub const IMAGEN_3: &str = "imagen-3.0-generate-001";
pub const IMAGEN_3_FAST: &str = "imagen-3.0-fast-001";

// ── Request types ──

#[derive(Serialize)]
struct PredictRequest {
    instances: Vec<Instance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<Parameters>,
}

#[derive(Serialize)]
struct Instance {
    prompt: String,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct Parameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    sampleCount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    aspectRatio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    personGeneration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safetyFilterLevel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    addWatermark: Option<bool>,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

// ── Response types ──

#[derive(Deserialize)]
struct PredictResponse {
    predictions: Vec<PredictResult>,
}

#[derive(Deserialize)]
struct PredictResult {
    #[serde(rename = "bytesBase64Encoded")]
    bytes_base64_encoded: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
}

/// A Gemini Imagen image generation model.
pub struct GeminiImageModel {
    api_key: SecretString,
    model_id: String,
    client: reqwest::Client,
    base_url: String,
}

impl GeminiImageModel {
    /// Create a new Gemini Imagen model.
    pub fn new(api_key: impl Into<String>, model_id: &str) -> Self {
        Self {
            api_key: SecretString::from(api_key.into()),
            model_id: model_id.to_string(),
            client: reqwest::Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Override the base URL.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    fn predict_url(&self) -> String {
        format!(
            "{}/models/{}:predict?key={}",
            self.base_url,
            self.model_id,
            self.api_key.expose_secret()
        )
    }
}

#[async_trait]
impl ImageModel for GeminiImageModel {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn provider_id(&self) -> &str {
        "gemini"
    }

    async fn generate_image(
        &self,
        prompt: &str,
        options: ImageGenerationOptions,
    ) -> AiResult<ImageResult> {
        let n = options.n.unwrap_or(1);

        let aspect_ratio: Option<String> = options
            .aspect_ratio
            .clone()
            .or_else(|| {
                // Derive from size if no explicit aspect_ratio
                options.size.as_deref().and_then(|s| {
                    let parts: Vec<&str> = s.split('x').collect();
                    if parts.len() == 2 {
                        Some(format!("{}:{}", parts[0], parts[1]))
                    } else {
                        None
                    }
                })
            });

        let mut extra = options.provider_options.clone().unwrap_or_default();

        // Move known extra params from provider_options or fill from options
        let person_generation = extra.remove("personGeneration")
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        let safety_filter_level = extra.remove("safetyFilterLevel")
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        let add_watermark = extra.remove("addWatermark")
            .and_then(|v| v.as_bool());

        let request = PredictRequest {
            instances: vec![Instance {
                prompt: prompt.to_string(),
            }],
            parameters: Some(Parameters {
                sampleCount: Some(n),
                aspectRatio: aspect_ratio,
                personGeneration: person_generation,
                safetyFilterLevel: safety_filter_level,
                addWatermark: add_watermark,
                extra,
            }),
        };

        let response = self
            .client
            .post(self.predict_url())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AiError::Transport {
                message: e.to_string(),
                source: Some(Box::new(e)),
            })?;

        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = match response.text().await {
                Ok(body) => body,
                Err(e) => {
                    tracing::warn!(status = status_code, error = %e, "Failed to read Imagen error response body");
                    format!("<failed to read response body: {e}>")
                }
            };
            return Err(AiError::ProviderError {
                provider: "gemini".to_string(),
                status: Some(status_code),
                message: body,
            });
        }

        let api_response: PredictResponse = response
            .json()
            .await
            .map_err(|e| AiError::Serialization(e.to_string()))?;

        let images: Vec<GeneratedFile> = api_response
            .predictions
            .into_iter()
            .map(|p| {
                let bytes = STANDARD
                    .decode(&p.bytes_base64_encoded)
                    .unwrap_or_default();
                GeneratedFile {
                    base64: p.bytes_base64_encoded,
                    bytes,
                    media_type: p.mime_type,
                }
            })
            .collect();

        let image = images.first().cloned().ok_or_else(|| {
            AiError::ProviderError {
                provider: "gemini".to_string(),
                status: None,
                message: "Imagen returned no predictions".to_string(),
            }
        })?;

        Ok(ImageResult {
            image,
            images,
            usage: Usage::default(),
            metadata: ResponseMetadata::default(),
        })
    }
}
