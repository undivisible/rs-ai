//! Integration tests for OpenAI-compatible provider presets

#[test]
fn test_openrouter_preset_config() {
    use rs_ai_openai_compatible::presets::openrouter;

    let config = openrouter::config("test-api-key");
    assert_eq!(config.base_url(), "https://openrouter.ai/api/v1");

    // Verify model IDs are available
    assert_eq!(openrouter::CLAUDE_SONNET, "anthropic/claude-3.5-sonnet");
    assert_eq!(openrouter::GPT4_TURBO, "openai/gpt-4-turbo");
    assert_eq!(openrouter::GEMINI_2_FLASH, "google/gemini-2.0-flash");
}

#[test]
fn test_kilo_preset_config() {
    use rs_ai_openai_compatible::presets::kilo;

    let config = kilo::config("test-api-key");
    assert_eq!(config.base_url(), "https://api.kilo.ai/api/gateway");

    assert_eq!(kilo::CLAUDE_SONNET, "claude-3-5-sonnet");
    assert_eq!(kilo::GPT4O, "gpt-4o");
}

#[test]
fn test_local_providers() {
    use rs_ai_openai_compatible::presets::{lm_studio, ollama, vllm};

    // Test default local configurations
    let ollama_config = ollama::config(None);
    assert_eq!(ollama_config.base_url(), "http://localhost:11434/v1");

    let vllm_config = vllm::config(None);
    assert_eq!(vllm_config.base_url(), "http://localhost:8000/v1");

    let lm_studio_config = lm_studio::config(None);
    assert_eq!(lm_studio_config.base_url(), "http://localhost:1234/v1");
}

#[test]
fn test_custom_endpoint_configuration() {
    use rs_ai_openai_compatible::presets::vllm;

    let custom_config = vllm::config(Some("http://custom-server:9000/v1"));
    assert_eq!(custom_config.base_url(), "http://custom-server:9000/v1");
}

#[test]
fn test_bedrock_preset() {
    use rs_ai_openai_compatible::presets::bedrock;

    let config = bedrock::config("us-west-2");
    assert!(config.base_url().contains("bedrock-runtime"));
    assert!(config.base_url().contains("us-west-2"));

    assert_eq!(
        bedrock::CLAUDE_SONNET_4_6,
        "anthropic.claude-sonnet-4-20250514"
    );
}

#[test]
fn test_together_preset() {
    use rs_ai_openai_compatible::presets::together;

    let config = together::config("test-api-key");
    assert_eq!(config.base_url(), "https://api.together.xyz/v1");

    assert_eq!(together::LLAMA_3_70B, "meta-llama/Llama-3-70b-chat-hf");
}

#[test]
fn test_all_presets_accessible() {
    use rs_ai_openai_compatible::presets;

    // Verify all preset modules exist and are accessible
    let _ = presets::openrouter::config("key");
    let _ = presets::kilo::config("key");
    let _ = presets::together::config("key");
    let _ = presets::octoml::config("key");
    let _ = presets::cloudflare::config("account", "token");
    let _ = presets::azure::config("resource", "key");
    let _ = presets::ollama::config(None);
    let _ = presets::vllm::config(None);
    let _ = presets::lm_studio::config(None);
    let _ = presets::text_gen_webui::config(None);

    // If we got here, all presets are accessible
    assert!(true);
}
