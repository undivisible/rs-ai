//! Pre-configured endpoints for popular OpenAI-compatible providers.
//!
//! This module provides convenient factory functions for connecting to common AI services.

use crate::config::OpenAiCompatibleConfig;

/// OpenRouter - Access to 300+ AI models through one unified API
pub mod openrouter {
    use super::*;

    /// Create configuration for OpenRouter
    pub fn config(api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://openrouter.ai/api/v1", api_key)
            .with_header("HTTP-Referer", "https://rai.example.com")
            .with_header("X-Title", "RAI SDK")
    }

    pub const CLAUDE_SONNET: &str = "anthropic/claude-3.5-sonnet";
    pub const CLAUDE_OPUS: &str = "anthropic/claude-3-opus";
    pub const GPT4_TURBO: &str = "openai/gpt-4-turbo";
    pub const GEMINI_2_FLASH: &str = "google/gemini-2.0-flash";
    pub const LLAMA_3_70B: &str = "meta-llama/llama-3-70b-instruct";
}

/// Amazon Bedrock - AWS-managed generative AI models
pub mod bedrock {
    use super::*;

    /// Create configuration for Amazon Bedrock
    /// Note: Bedrock uses AWS_REGION and AWS credentials from environment
    pub fn config(region: &str) -> OpenAiCompatibleConfig {
        let base_url = format!("https://bedrock-runtime.{}.amazonaws.com", region);
        OpenAiCompatibleConfig::new(
            base_url,
            "", // Will use AWS credentials
        )
    }

    pub const CLAUDE_SONNET_4_6: &str = "anthropic.claude-sonnet-4-20250514";
    pub const CLAUDE_OPUS_4: &str = "anthropic.claude-3-opus-20240229";
    pub const CLAUDE_HAIKU: &str = "anthropic.claude-3-haiku-20240307";
}

/// Kilo AI Gateway - Unified routing for hundreds of models
pub mod kilo {
    use super::*;

    /// Create configuration for Kilo AI Gateway
    pub fn config(api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://api.kilo.ai/api/gateway", api_key)
    }

    pub const CLAUDE_SONNET: &str = "claude-3-5-sonnet";
    pub const GPT4O: &str = "gpt-4o";
    pub const GEMINI_2_FLASH: &str = "gemini-2.0-flash";
}

/// Together AI - Scale with open-source and custom models
pub mod together {
    use super::*;

    /// Create configuration for Together AI
    pub fn config(api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://api.together.xyz/v1", api_key)
    }

    pub const LLAMA_3_70B: &str = "meta-llama/Llama-3-70b-chat-hf";
    pub const MISTRAL_7B: &str = "mistralai/Mistral-7B-Instruct-v0.2";
}

/// OctoML - Optimized open-source models on serverless infrastructure
pub mod octoml {
    use super::*;

    /// Create configuration for OctoML
    pub fn config(api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://text.octoml.cloud/v1", api_key)
    }

    pub const LLAMA_3_8B: &str = "meta-llama-3-8b-instruct";
    pub const MISTRAL_7B: &str = "mistral-7b-instruct";
}

/// Azure OpenAI - Microsoft's hosted OpenAI models
pub mod azure {
    use super::*;

    /// Create configuration for Azure OpenAI
    pub fn config(resource_name: &str, api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        let api_key_str = api_key.into();
        let base_url = format!(
            "https://{}.openai.azure.com/openai/deployments",
            resource_name
        );
        OpenAiCompatibleConfig::new(base_url, api_key_str.clone())
            .with_header("api-key", api_key_str)
    }

    pub const GPT4_TURBO: &str = "gpt-4-turbo";
    pub const GPT35_TURBO: &str = "gpt-3.5-turbo";
}

/// Cloudflare Workers AI - Run AI models on Cloudflare's edge network
pub mod cloudflare {
    use super::*;

    /// Create configuration for Cloudflare Workers AI
    pub fn config(account_id: &str, api_token: impl Into<String>) -> OpenAiCompatibleConfig {
        let base_url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/ai",
            account_id
        );
        OpenAiCompatibleConfig::new(base_url, api_token)
    }

    pub const LLAMA_2_7B: &str = "@cf/meta/llama-2-7b-chat-fp16";
    pub const MISTRAL_7B: &str = "@cf/mistral/mistral-7b-instruct-v0.1";
}

/// vLLM - Optimized inference engine for large language models
pub mod vllm {
    use super::*;

    /// Create configuration for vLLM (self-hosted)
    /// Default assumes vLLM running on localhost:8000
    pub fn config(base_url: Option<&str>) -> OpenAiCompatibleConfig {
        let url = base_url.unwrap_or("http://localhost:8000/v1");
        OpenAiCompatibleConfig::new(url, "not-needed")
    }

    pub const CUSTOM_MODEL: &str = "custom-model"; // Use your deployed model name
}

/// Ollama - Run language models locally
pub mod ollama {
    use super::*;

    /// Create configuration for Ollama (self-hosted)
    /// Default assumes Ollama running on localhost:11434
    pub fn config(base_url: Option<&str>) -> OpenAiCompatibleConfig {
        let url = base_url.unwrap_or("http://localhost:11434/v1");
        OpenAiCompatibleConfig::new(url, "not-needed")
    }

    pub const LLAMA_2: &str = "llama2";
    pub const MISTRAL: &str = "mistral";
    pub const NEURAL_CHAT: &str = "neural-chat";
}

/// LM Studio - Run models locally with a simple UI
pub mod lm_studio {
    use super::*;

    /// Create configuration for LM Studio (self-hosted)
    /// Default assumes LM Studio running on localhost:1234
    pub fn config(base_url: Option<&str>) -> OpenAiCompatibleConfig {
        let url = base_url.unwrap_or("http://localhost:1234/v1");
        OpenAiCompatibleConfig::new(url, "not-needed")
    }

    pub const CUSTOM_MODEL: &str = "local-model"; // Use your loaded model name
}

/// text-generation-webui - A Gradio web UI for LLMs
pub mod text_gen_webui {
    use super::*;

    /// Create configuration for text-generation-webui (self-hosted)
    /// Default assumes it running on localhost:5000
    pub fn config(base_url: Option<&str>) -> OpenAiCompatibleConfig {
        let url = base_url.unwrap_or("http://localhost:5000/v1");
        OpenAiCompatibleConfig::new(url, "not-needed")
    }

    pub const CUSTOM_MODEL: &str = "model"; // Use your loaded model name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_config() {
        let config = openrouter::config("test-key");
        assert_eq!(config.base_url(), "https://openrouter.ai/api/v1");
    }

    #[test]
    fn test_kilo_config() {
        let config = kilo::config("test-key");
        assert_eq!(config.base_url(), "https://api.kilo.ai/api/gateway");
    }

    #[test]
    fn test_ollama_local() {
        let config = ollama::config(None);
        assert_eq!(config.base_url(), "http://localhost:11434/v1");
    }

    #[test]
    fn test_custom_vllm_endpoint() {
        let config = vllm::config(Some("http://my-vllm-server:8000/v1"));
        assert_eq!(config.base_url(), "http://my-vllm-server:8000/v1");
    }
}
