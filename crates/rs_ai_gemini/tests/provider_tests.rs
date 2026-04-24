use rs_ai_ai::LanguageModel;
use rs_ai_gemini::GeminiProvider;

#[test]
fn test_gemini_provider_creation() {
    let provider = GeminiProvider::new("test-api-key");
    let model = provider.model("gemini-2.5-flash");

    assert_eq!(model.model_id(), "gemini-2.5-flash");
    assert_eq!(model.provider_id(), "gemini");
}

#[test]
fn test_gemini_flash_convenience() {
    let provider = GeminiProvider::new("test-api-key");
    let model = provider.gemini_flash();

    assert_eq!(model.model_id(), "gemini-2.5-flash");
    assert_eq!(model.provider_id(), "gemini");
}

#[test]
fn test_gemini_pro_convenience() {
    let provider = GeminiProvider::new("test-api-key");
    let model = provider.gemini_pro();

    assert_eq!(model.model_id(), "gemini-2.5-pro");
}

#[test]
fn test_gemini_flash_lite_convenience() {
    let provider = GeminiProvider::new("test-api-key");
    let model = provider.gemini_flash_lite();

    assert_eq!(model.model_id(), "gemini-2.5-flash-lite");
}

#[test]
fn test_gemini_3_flash_convenience() {
    let provider = GeminiProvider::new("test-api-key");
    let model = provider.gemini_3_flash();

    assert_eq!(model.model_id(), "gemini-3-flash");
}

#[test]
fn test_gemini_31_pro_convenience() {
    let provider = GeminiProvider::new("test-api-key");
    let model = provider.gemini_31_pro();

    assert_eq!(model.model_id(), "gemini-3.1-pro-preview");
}

#[test]
fn test_gemini_model_with_custom_base_url() {
    let provider = GeminiProvider::new("test-api-key");
    let model = provider.model_with_base_url("gemini-2.5-flash", "https://custom.googleapis.com");

    assert_eq!(model.model_id(), "gemini-2.5-flash");
    assert_eq!(model.provider_id(), "gemini");
}

#[test]
fn test_gemini_model_capabilities() {
    let provider = GeminiProvider::new("test-api-key");
    let model = provider.model("gemini-2.5-flash");

    let capabilities = model.capabilities();
    assert!(capabilities.supports_all(&[rs_ai_ai::Capability::TextInput]));
    assert!(capabilities.supports_all(&[rs_ai_ai::Capability::TextOutput]));
}
