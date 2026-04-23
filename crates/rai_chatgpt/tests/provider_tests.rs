use rai_ai::LanguageModel;
use rai_chatgpt::ChatGptProvider;

#[test]
fn test_chatgpt_provider_creation() {
    let provider = ChatGptProvider::new("sk-test-key");
    let model = provider.model("gpt-4o");

    assert_eq!(model.model_id(), "gpt-4o");
    assert_eq!(model.provider_id(), "chatgpt");
}

#[test]
fn test_chatgpt_gpt4o_convenience() {
    let provider = ChatGptProvider::new("sk-test-key");
    let model = provider.gpt4o();

    assert_eq!(model.model_id(), "gpt-4o");
    assert_eq!(model.provider_id(), "chatgpt");
}

#[test]
fn test_chatgpt_gpt4o_mini_convenience() {
    let provider = ChatGptProvider::new("sk-test-key");
    let model = provider.gpt4o_mini();

    assert_eq!(model.model_id(), "gpt-4o-mini");
}

#[test]
fn test_chatgpt_gpt54_convenience() {
    let provider = ChatGptProvider::new("sk-test-key");
    let model = provider.gpt54();

    assert_eq!(model.model_id(), "gpt-5.4");
}

#[test]
fn test_chatgpt_gpt54_mini_convenience() {
    let provider = ChatGptProvider::new("sk-test-key");
    let model = provider.gpt54_mini();

    assert_eq!(model.model_id(), "gpt-5.4-mini");
}

#[test]
fn test_chatgpt_gpt54_nano_convenience() {
    let provider = ChatGptProvider::new("sk-test-key");
    let model = provider.gpt54_nano();

    assert_eq!(model.model_id(), "gpt-5.4-nano");
}

#[test]
fn test_chatgpt_with_org_id() {
    let provider = ChatGptProvider::new("sk-test-key").with_org("org-12345");

    let model = provider.gpt4o();
    assert_eq!(model.model_id(), "gpt-4o");
    assert_eq!(model.provider_id(), "chatgpt");
}

#[test]
fn test_chatgpt_model_capabilities() {
    let provider = ChatGptProvider::new("sk-test-key");
    let model = provider.gpt4o();

    let capabilities = model.capabilities();
    assert!(capabilities.supports_all(&[rai_ai::Capability::TextInput]));
    assert!(capabilities.supports_all(&[rai_ai::Capability::TextOutput]));
    assert!(capabilities.supports_all(&[rai_ai::Capability::Streaming]));
}