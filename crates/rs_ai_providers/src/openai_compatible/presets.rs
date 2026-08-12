//! Pre-configured endpoints for popular OpenAI-compatible providers.
//!
//! This module provides convenient factory functions for connecting to common AI services.

use super::config::OpenAiCompatibleConfig;

/// OpenRouter - Access to 300+ AI models through one unified API
pub mod openrouter {
    use super::*;

    /// Create configuration for OpenRouter
    pub fn config(api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://openrouter.ai/api/v1", api_key)
            .with_header("HTTP-Referer", "https://rai.example.com")
            .with_header("X-Title", "RAI SDK")
    }

    /// Create a provider that can refresh its model metadata from OpenRouter.
    ///
    /// ```no_run
    /// # async fn example() -> rs_ai_core::AiResult<()> {
    /// use rs_ai_providers::openai_compatible::presets::openrouter;
    ///
    /// let mut provider = openrouter::provider("sk-or-...");
    /// let update = provider.refresh_models().await?;
    /// let model = provider.language_model("anthropic/claude-sonnet-4-6");
    /// assert!(model.capabilities().iter().next().is_some());
    /// # let _ = update;
    /// # Ok(())
    /// # }
    pub fn provider(api_key: impl Into<String>) -> super::super::OpenAiCompatibleProvider {
        super::super::OpenAiCompatibleProvider::new(config(api_key), "openrouter", "OpenRouter")
    }

    /// Claude 3.5 Sonnet model identifier.
    pub const CLAUDE_SONNET: &str = "anthropic/claude-3.5-sonnet";
    /// Claude 3 Opus model identifier.
    pub const CLAUDE_OPUS: &str = "anthropic/claude-3-opus";
    /// GPT-4 Turbo model identifier.
    pub const GPT4_TURBO: &str = "openai/gpt-4-turbo";
    /// Gemini 2.0 Flash model identifier.
    pub const GEMINI_2_FLASH: &str = "google/gemini-2.0-flash";
    /// Llama 3 70B model identifier.
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
            base_url, "", // Will use AWS credentials
        )
    }

    /// Claude Sonnet 4.6 model identifier.
    pub const CLAUDE_SONNET_4_6: &str = "anthropic.claude-sonnet-4-20250514";
    /// Claude 3 Opus model identifier.
    pub const CLAUDE_OPUS_4: &str = "anthropic.claude-3-opus-20240229";
    /// Claude 3 Haiku model identifier.
    pub const CLAUDE_HAIKU: &str = "anthropic.claude-3-haiku-20240307";
}

/// Kilo AI Gateway - Unified routing for hundreds of models
pub mod kilo {
    use super::*;

    /// Create configuration for Kilo AI Gateway
    pub fn config(api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://api.kilo.ai/api/gateway", api_key)
    }

    /// Claude 3.5 Sonnet model identifier.
    pub const CLAUDE_SONNET: &str = "claude-3-5-sonnet";
    /// GPT-4o model identifier.
    pub const GPT4O: &str = "gpt-4o";
    /// Gemini 2.0 Flash model identifier.
    pub const GEMINI_2_FLASH: &str = "gemini-2.0-flash";
}

/// Together AI - Scale with open-source and custom models
pub mod together {
    use super::*;

    /// Create configuration for Together AI
    pub fn config(api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://api.together.xyz/v1", api_key)
    }

    /// Llama 3 70B model identifier.
    pub const LLAMA_3_70B: &str = "meta-llama/Llama-3-70b-chat-hf";
    /// Mistral 7B model identifier.
    pub const MISTRAL_7B: &str = "mistralai/Mistral-7B-Instruct-v0.2";
}

/// OctoML - Optimized open-source models on serverless infrastructure
pub mod octoml {
    use super::*;

    /// Create configuration for OctoML
    pub fn config(api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://text.octoml.cloud/v1", api_key)
    }

    /// Llama 3 8B model identifier.
    pub const LLAMA_3_8B: &str = "meta-llama-3-8b-instruct";
    /// Mistral 7B model identifier.
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

    /// GPT-4 Turbo model identifier.
    pub const GPT4_TURBO: &str = "gpt-4-turbo";
    /// GPT-3.5 Turbo model identifier.
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

    /// Llama 2 7B model identifier.
    pub const LLAMA_2_7B: &str = "@cf/meta/llama-2-7b-chat-fp16";
    /// Mistral 7B model identifier.
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

    /// Custom model placeholder — use your deployed model name.
    pub const CUSTOM_MODEL: &str = "custom-model";
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

    /// Llama 2 model identifier.
    pub const LLAMA_2: &str = "llama2";
    /// Mistral model identifier.
    pub const MISTRAL: &str = "mistral";
    /// Neural Chat model identifier.
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

    /// Custom model placeholder — use your loaded model name.
    pub const CUSTOM_MODEL: &str = "local-model";
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

    /// Custom model placeholder — use your loaded model name.
    pub const CUSTOM_MODEL: &str = "model";
}

/// Anthropic — Anthropic's Claude models via their API gateway
/// Note: Anthropic uses a non-OpenAI format natively. This preset works
/// through Anthropic's OpenAI-compatible gateway or third-party gateways.
pub mod anthropic {
    use super::*;

    /// Create configuration for Anthropic's OpenAI-compatible endpoint.
    pub fn config(api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://api.anthropic.com/v1", api_key)
            .with_header("anthropic-version", "2023-06-01")
    }

    /// Claude Sonnet 4.6 model identifier.
    pub const CLAUDE_SONNET_4: &str = "claude-sonnet-4-20250514";
    /// Claude 3.5 Haiku model identifier.
    pub const CLAUDE_HAIKU: &str = "claude-3-5-haiku-20241022";
    /// Claude 3 Opus model identifier.
    pub const CLAUDE_OPUS: &str = "claude-opus-4-20250514";
}

/// Groq — Ultra-fast LPU inference for open-source models
pub mod groq {
    use super::*;

    /// Create configuration for Groq.
    pub fn config(api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://api.groq.com/openai/v1", api_key)
    }

    /// Llama 3.3 70B model identifier.
    pub const LLAMA_3_3_70B: &str = "llama-3.3-70b-versatile";
    /// Llama 3.1 8B model identifier.
    pub const LLAMA_3_1_8B: &str = "llama-3.1-8b-instant";
    /// Mixtral 8x7B model identifier.
    pub const MIXTRAL_8X7B: &str = "mixtral-8x7b-32768";
    /// DeepSeek R1 distilled Llama 70B model identifier.
    pub const DEEPSEEK_R1: &str = "deepseek-r1-distill-llama-70b";
}

/// DeepSeek — Powerful open-weight models
pub mod deepseek {
    use super::*;

    /// Create configuration for DeepSeek.
    pub fn config(api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://api.deepseek.com/v1", api_key)
            .with_header("HTTP-Referer", "https://rai.example.com")
    }

    /// DeepSeek Chat model identifier.
    pub const CHAT: &str = "deepseek-chat";
    /// DeepSeek Reasoner model identifier.
    pub const REASONER: &str = "deepseek-reasoner";
}

/// Mistral AI — Open-weight European models
pub mod mistral {
    use super::*;

    /// Create configuration for Mistral AI.
    pub fn config(api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://api.mistral.ai/v1", api_key)
    }

    /// Mistral Large model identifier.
    pub const LARGE: &str = "mistral-large-latest";
    /// Mistral Small model identifier.
    pub const SMALL: &str = "mistral-small-latest";
    /// Codestral model identifier.
    pub const CODESTRAL: &str = "codestral-latest";
}

/// Perplexity AI — Search-augmented language models
pub mod perplexity {
    use super::*;

    /// Create configuration for Perplexity AI.
    pub fn config(api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://api.perplexity.ai", api_key)
    }

    /// Sonar Pro model identifier.
    pub const SONAR_PRO: &str = "sonar-pro";
    /// Sonar model identifier.
    pub const SONAR: &str = "sonar";
}

/// Fireworks AI — Fast inference for open-source models
pub mod fireworks {
    use super::*;

    /// Create configuration for Fireworks AI.
    pub fn config(api_key: impl Into<String>) -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://api.fireworks.ai/inference/v1", api_key)
    }

    /// Llama 3.3 70B model identifier.
    pub const LLAMA_3_3_70B: &str = "accounts/fireworks/models/llama-v3p3-70b-instruct";
    /// Firefunction v2 model identifier.
    pub const FIREFUNCTION_V2: &str = "accounts/fireworks/models/firefunction-v2";
    /// DeepSeek R1 model identifier.
    pub const DEEPSEEK_R1: &str = "accounts/fireworks/models/deepseek-r1";
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

    #[test]
    fn test_anthropic_config() {
        let config = anthropic::config("test-key");
        assert_eq!(config.base_url(), "https://api.anthropic.com/v1");
    }

    #[test]
    fn test_groq_config() {
        let config = groq::config("test-key");
        assert_eq!(config.base_url(), "https://api.groq.com/openai/v1");
    }

    #[test]
    fn test_deepseek_config() {
        let config = deepseek::config("test-key");
        assert_eq!(config.base_url(), "https://api.deepseek.com/v1");
    }

    #[test]
    fn test_mistral_config() {
        let config = mistral::config("test-key");
        assert_eq!(config.base_url(), "https://api.mistral.ai/v1");
    }

    #[test]
    fn test_perplexity_config() {
        let config = perplexity::config("test-key");
        assert_eq!(config.base_url(), "https://api.perplexity.ai");
    }

    #[test]
    fn test_fireworks_config() {
        let config = fireworks::config("test-key");
        assert_eq!(config.base_url(), "https://api.fireworks.ai/inference/v1");
    }
}
