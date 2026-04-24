use rs_ai_ai::LanguageModel;
use rs_ai_portkey::PortkeyProvider;

#[test]
fn test_provider_creation() {
    let provider = PortkeyProvider::new("test-key");
    let model = provider.model("gpt-4");

    assert_eq!(model.model_id(), "gpt-4");
    assert_eq!(model.provider_id(), "portkey");
}

#[test]
fn test_provider_with_custom_url() {
    let provider = PortkeyProvider::new("test-key").with_base_url("https://custom.portkey.ai/v1");

    let model = provider.model("gpt-4");
    assert_eq!(model.model_id(), "gpt-4");
}

#[test]
fn test_model_capabilities() {
    let provider = PortkeyProvider::new("test-key");
    let model = provider.model("gpt-4");

    let capabilities = model.capabilities();
    assert!(capabilities.supports_all(&[rs_ai_ai::Capability::TextInput]));
    assert!(capabilities.supports_all(&[rs_ai_ai::Capability::TextOutput]));
    assert!(capabilities.supports_all(&[rs_ai_ai::Capability::Streaming]));
    assert!(capabilities.supports_all(&[rs_ai_ai::Capability::ToolCalling]));
}

#[test]
fn test_multiple_models_same_provider() {
    let provider = PortkeyProvider::new("test-key");

    let model1 = provider.model("gpt-4");
    let model2 = provider.model("gpt-4o");
    let model3 = provider.model("claude-3-sonnet");

    assert_eq!(model1.model_id(), "gpt-4");
    assert_eq!(model2.model_id(), "gpt-4o");
    assert_eq!(model3.model_id(), "claude-3-sonnet");
    assert_eq!(model1.provider_id(), model2.provider_id());
    assert_eq!(model1.provider_id(), model3.provider_id());
}

#[test]
fn test_provider_is_cloneable() {
    let provider1 = PortkeyProvider::new("test-key");
    let provider2 = provider1.clone();

    let model1 = provider1.model("gpt-4");
    let model2 = provider2.model("gpt-4");

    assert_eq!(model1.model_id(), model2.model_id());
}
