use rs_ai_claude::ClaudeProvider;
use rs_ai_traits::{LanguageModel, Provider};

#[test]
fn test_claude_provider_creation() {
    let provider = ClaudeProvider::new("test-api-key");
    let model = provider.model("claude-sonnet-4-6");

    assert_eq!(model.model_id(), "claude-sonnet-4-6");
    assert_eq!(model.provider_id(), "anthropic");
}

#[test]
fn test_claude_provider_id() {
    let provider = ClaudeProvider::new("test-api-key");
    assert_eq!(provider.id(), "anthropic");
    assert_eq!(provider.name(), "Anthropic");
}

#[test]
fn test_claude_sonnet_convenience() {
    let provider = ClaudeProvider::new("test-api-key");
    let model = provider.claude_sonnet();

    assert_eq!(model.model_id(), "claude-sonnet-4-6");
    assert_eq!(model.provider_id(), "anthropic");
}

#[test]
fn test_claude_opus_convenience() {
    let provider = ClaudeProvider::new("test-api-key");
    let model = provider.claude_opus();

    assert_eq!(model.model_id(), "claude-opus-4-6");
}

#[test]
fn test_claude_haiku_convenience() {
    let provider = ClaudeProvider::new("test-api-key");
    let model = provider.claude_haiku();

    assert_eq!(model.model_id(), "claude-haiku-4-5-20251001");
}

#[test]
fn test_claude_with_custom_base_url() {
    let provider =
        ClaudeProvider::new("test-api-key").with_base_url("https://custom.anthropic.com");

    let model = provider.model("claude-sonnet-4-6");
    assert_eq!(model.provider_id(), "anthropic");
}

#[test]
fn test_claude_language_model_trait() {
    let provider = ClaudeProvider::new("test-api-key");
    let model_result = provider.language_model("claude-sonnet-4-6");

    assert!(model_result.is_ok());
    let model = model_result.unwrap();
    assert_eq!(model.model_id(), "claude-sonnet-4-6");
    assert_eq!(model.provider_id(), "anthropic");
}

#[test]
fn test_claude_model_capabilities() {
    let provider = ClaudeProvider::new("test-api-key");
    let model = provider.claude_sonnet();

    let capabilities = model.capabilities();
    assert!(capabilities.supports_all(&[rs_ai_traits::Capability::TextInput]));
    assert!(capabilities.supports_all(&[rs_ai_traits::Capability::TextOutput]));
    assert!(capabilities.supports_all(&[rs_ai_traits::Capability::Streaming]));
}
