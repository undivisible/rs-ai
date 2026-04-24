use rs_ai_ollama::OllamaProvider;
use rs_ai_traits::{LanguageModel, Provider};

#[test]
fn test_ollama_provider_creation() {
    let provider = OllamaProvider::new();
    assert_eq!(provider.id(), "ollama");
    assert_eq!(provider.name(), "Ollama");
}

#[test]
fn test_ollama_provider_with_custom_url() {
    let provider = OllamaProvider::new().with_base_url("http://custom-ollama:11434");

    assert_eq!(provider.id(), "ollama");
}

#[test]
fn test_ollama_default_provider() {
    let provider = OllamaProvider::default();
    assert_eq!(provider.id(), "ollama");
    assert_eq!(provider.name(), "Ollama");
}

#[test]
fn test_ollama_model_creation() {
    let provider = OllamaProvider::new();
    let model = provider.model("llama3");

    assert_eq!(model.model_id(), "llama3");
    assert_eq!(model.provider_id(), "ollama");
}

#[test]
fn test_ollama_language_model_trait() {
    let provider = OllamaProvider::new();
    let model_result = provider.language_model("mistral");

    assert!(model_result.is_ok());
    let model = model_result.unwrap();
    assert_eq!(model.model_id(), "mistral");
    assert_eq!(model.provider_id(), "ollama");
}

#[test]
fn test_ollama_embedding_model_trait() {
    let provider = OllamaProvider::new();
    let model_result = provider.embedding_model("nomic-embed-text");

    assert!(model_result.is_ok());
}

#[test]
fn test_ollama_available_models_empty() {
    let provider = OllamaProvider::new();
    let models = provider.available_models();

    // OllamaProvider returns empty list for sync available_models()
    assert!(models.is_empty());
}
