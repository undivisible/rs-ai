use super::client::CloudflareClient;
use super::model::CloudflareModel;
use crate::CloudflareModelId;

/// Cloudflare Workers AI provider for creating models.
#[derive(Clone)]
pub struct CloudflareProvider {
    client: std::sync::Arc<CloudflareClient>,
}

impl CloudflareProvider {
    /// Create a new Cloudflare provider with the given account ID and API token.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let provider = CloudflareProvider::new("your-account-id", "your-api-token");
    /// let model = provider.llama_3_1_8b_instruct();
    /// ```
    pub fn new(account_id: impl Into<String>, api_token: impl Into<String>) -> Self {
        let client = CloudflareClient::new(account_id, api_token);
        Self {
            client: std::sync::Arc::new(client),
        }
    }

    /// Create a model instance for the given model ID string.
    pub fn model(&self, model_id: &str) -> CloudflareModel {
        CloudflareModel::new(model_id.to_string(), self.client.clone())
    }

    /// Llama 3.1 8B Instruct — fast, lightweight general-purpose model.
    pub fn llama_3_1_8b_instruct(&self) -> CloudflareModel {
        self.model(CloudflareModelId::Llama31_8bInstruct.as_str())
    }

    /// Llama 3.3 70B Instruct FP8 Fast — high-quality model optimised for speed.
    pub fn llama_3_3_70b_instruct_fp8_fast(&self) -> CloudflareModel {
        self.model(CloudflareModelId::Llama33_70bInstructFp8Fast.as_str())
    }

    /// Mistral 7B Instruct v0.1 — compact instruction-following model.
    pub fn mistral_7b_instruct(&self) -> CloudflareModel {
        self.model(CloudflareModelId::Mistral7bInstructV01.as_str())
    }

    /// Gemma 7B IT — Google's Gemma 7B instruction-tuned model.
    pub fn gemma_7b_it(&self) -> CloudflareModel {
        self.model(CloudflareModelId::Gemma7bIt.as_str())
    }

    /// Qwen 2.5 Coder 7B Instruct — code-focused model from Alibaba.
    pub fn qwen2_5_coder_7b_instruct(&self) -> CloudflareModel {
        self.model(CloudflareModelId::Qwen25Coder7bInstruct.as_str())
    }
}
