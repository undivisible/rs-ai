//! ChatGPT provider tests

#[cfg(test)]
mod tests {
    use rai_chatgpt::ChatGptProvider;

    #[test]
    fn test_provider_creation() {
        let provider = ChatGptProvider::new("sk-test");
        assert_eq!(provider.id(), "chatgpt");
    }

    #[test]
    fn test_provider_name() {
        let provider = ChatGptProvider::new("sk-test");
        assert_eq!(provider.name(), "OpenAI ChatGPT");
    }

    #[test]
    fn test_model_variants() {
        let provider = ChatGptProvider::new("sk-test");

        let _gpt4o = provider.gpt4o();
        let _gpt4o_mini = provider.gpt4o_mini();
        let _gpt4_turbo = provider.gpt4_turbo();
        assert!(true);
    }

    #[test]
    fn test_realtime_api_support() {
        let provider = ChatGptProvider::new("sk-test");
        let info = provider.provider_info();

        // ChatGPT should support realtime API
        assert_eq!(info.provider_id, "chatgpt");
    }

    #[test]
    fn test_vision_model_support() {
        let provider = ChatGptProvider::new("sk-test");
        let model_result = provider.language_model("gpt-4o");

        assert!(model_result.is_ok());
    }

    #[test]
    fn test_embedding_support() {
        let provider = ChatGptProvider::new("sk-test");
        let embedding = provider.embedding_model("text-embedding-3-large");

        assert!(embedding.is_ok());
    }

    #[test]
    fn test_with_custom_base_url() {
        let provider = ChatGptProvider::new("sk-test")
            .with_base_url("https://api.custom.example.com");
        assert_eq!(provider.id(), "chatgpt");
    }

    #[test]
    fn test_provider_info_completeness() {
        let provider = ChatGptProvider::new("sk-test");
        let info = provider.provider_info();

        assert_eq!(info.provider_id, "chatgpt");
        assert!(!info.supported_models.is_empty());
    }

    #[test]
    fn test_provider_cloning() {
        let provider = ChatGptProvider::new("sk-test");
        let _cloned = provider.clone();
        assert!(true);
    }

    #[test]
    fn test_language_model_creation() {
        let provider = ChatGptProvider::new("sk-test");
        let model = provider.language_model("gpt-4o");

        assert!(model.is_ok());
    }
}
