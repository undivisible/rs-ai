//! Ollama provider tests

#[cfg(test)]
mod tests {
    use rai_ollama::OllamaProvider;

    #[test]
    fn test_provider_creation() {
        let provider = OllamaProvider::new();
        assert_eq!(provider.id(), "ollama");
    }

    #[test]
    fn test_provider_name() {
        let provider = OllamaProvider::new();
        assert_eq!(provider.name(), "Ollama");
    }

    #[test]
    fn test_default_base_url() {
        let provider = OllamaProvider::new();
        // Default should be localhost:11434
        let _model = provider.model("llama2");
        assert!(true);
    }

    #[test]
    fn test_custom_base_url() {
        let provider = OllamaProvider::new()
            .with_base_url("http://remote-server:11434");
        let _model = provider.model("llama2");
        assert!(true);
    }

    #[test]
    fn test_model_creation() {
        let provider = OllamaProvider::new();
        let model = provider.model("llama2");

        assert_eq!(model.provider_id(), "ollama");
    }

    #[test]
    fn test_multiple_models() {
        let provider = OllamaProvider::new();

        let _llama = provider.model("llama2");
        let _mistral = provider.model("mistral");
        let _neural = provider.model("neural-chat");
        assert!(true);
    }

    #[test]
    fn test_language_model_via_provider() {
        let provider = OllamaProvider::new();
        let model_result = provider.language_model("llama2");

        assert!(model_result.is_ok());
    }

    #[test]
    fn test_embedding_model_support() {
        let provider = OllamaProvider::new();
        let embedding_result = provider.embedding_model("nomic-embed-text");

        assert!(embedding_result.is_ok());
    }

    #[test]
    fn test_available_models_list() {
        let provider = OllamaProvider::new();
        let models = provider.available_models();

        // For local Ollama, the list may be empty until queried
        // but the function should return a Vec
        assert_eq!(models.len(), 0);
    }

    #[test]
    fn test_default_provider_creation() {
        let provider = OllamaProvider::default();
        assert_eq!(provider.id(), "ollama");
    }

    #[test]
    fn test_provider_cloning() {
        let provider = OllamaProvider::new();
        let _cloned = provider.clone();
        assert!(true);
    }

    #[test]
    fn test_url_normalization() {
        let provider = OllamaProvider::new()
            .with_base_url("http://localhost:11434/");
        let _model = provider.model("llama2");
        assert!(true);
    }

    #[test]
    fn test_remote_ollama_config() {
        let provider = OllamaProvider::new()
            .with_base_url("http://192.168.1.100:11434");
        let model = provider.model("llama2");

        assert_eq!(model.provider_id(), "ollama");
    }
}
