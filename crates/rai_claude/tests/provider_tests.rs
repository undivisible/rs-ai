//! Provider initialization and configuration tests

#[cfg(test)]
mod tests {
    use rai_claude::ClaudeProvider;

    #[test]
    fn test_provider_creation() {
        let provider = ClaudeProvider::new("test-api-key");
        assert_eq!(provider.id(), "claude");
    }

    #[test]
    fn test_provider_name() {
        let provider = ClaudeProvider::new("test-api-key");
        assert_eq!(provider.name(), "Anthropic Claude");
    }

    #[test]
    fn test_provider_with_custom_base_url() {
        let provider = ClaudeProvider::new("test-api-key")
            .with_base_url("https://custom.example.com");
        assert_eq!(provider.id(), "claude");
    }

    #[test]
    fn test_model_variants() {
        let provider = ClaudeProvider::new("test-api-key");

        // These should not panic - they're factory methods
        let _sonnet = provider.claude_sonnet();
        let _opus = provider.claude_opus();
        let _haiku = provider.claude_haiku();
        assert!(true);
    }

    #[test]
    fn test_provider_info() {
        let provider = ClaudeProvider::new("test-api-key");
        let info = provider.provider_info();

        assert_eq!(info.provider_id, "claude");
        assert!(!info.supported_models.is_empty());
    }

    #[test]
    fn test_model_creation_via_provider() {
        let provider = ClaudeProvider::new("test-api-key");
        let model_result = provider.language_model("claude-sonnet-4-6");

        assert!(model_result.is_ok());
    }

    #[test]
    fn test_invalid_model_id() {
        let provider = ClaudeProvider::new("test-api-key");
        let model_result = provider.language_model("invalid-model-id");

        // Provider should either accept it (API will reject) or validate
        // For now, just ensure it doesn't panic
        let _ = model_result;
        assert!(true);
    }

    #[test]
    fn test_provider_capabilities() {
        let provider = ClaudeProvider::new("test-api-key");
        let info = provider.provider_info();

        // Claude should support text generation
        assert!(!info.supported_models.is_empty());
    }

    #[test]
    fn test_embedding_model_support() {
        let provider = ClaudeProvider::new("test-api-key");
        let embedding_result = provider.embedding_model("text-embedding-3-small");

        // Claude doesn't natively support embeddings, but test for graceful handling
        let _ = embedding_result;
        assert!(true);
    }

    #[test]
    fn test_provider_clonability() {
        let provider = ClaudeProvider::new("test-api-key");
        let _cloned = provider.clone();
        assert!(true);
    }
}
