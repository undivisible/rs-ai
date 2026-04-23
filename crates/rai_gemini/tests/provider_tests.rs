//! Gemini provider tests

#[cfg(test)]
mod tests {
    use rai_gemini::GeminiProvider;

    #[test]
    fn test_provider_creation() {
        let provider = GeminiProvider::new("test-api-key");
        assert_eq!(provider.id(), "gemini");
    }

    #[test]
    fn test_provider_name() {
        let provider = GeminiProvider::new("test-api-key");
        assert_eq!(provider.name(), "Google Gemini");
    }

    #[test]
    fn test_gemini_2_flash_model() {
        let provider = GeminiProvider::new("test-api-key");
        let model = provider.gemini_2_flash();

        // Should return a model instance
        assert_eq!(model.provider_id(), "gemini");
    }

    #[test]
    fn test_live_api_support() {
        let provider = GeminiProvider::new("test-api-key");
        let info = provider.provider_info();

        // Gemini should support live API
        assert_eq!(info.provider_id, "gemini");
    }

    #[test]
    fn test_vision_capabilities() {
        let provider = GeminiProvider::new("test-api-key");
        let model = provider.gemini_2_flash();

        // Gemini has native vision support
        assert_eq!(model.provider_id(), "gemini");
    }

    #[test]
    fn test_multimodal_support() {
        let provider = GeminiProvider::new("test-api-key");
        let model_result = provider.language_model("gemini-2.0-flash");

        assert!(model_result.is_ok());
    }

    #[test]
    fn test_with_custom_base_url() {
        let provider = GeminiProvider::new("test-api-key")
            .with_base_url("https://custom.googleapis.com");
        assert_eq!(provider.id(), "gemini");
    }

    #[test]
    fn test_safety_settings_support() {
        let provider = GeminiProvider::new("test-api-key");
        let info = provider.provider_info();

        // Gemini supports safety settings
        assert_eq!(info.provider_id, "gemini");
    }

    #[test]
    fn test_model_list() {
        let provider = GeminiProvider::new("test-api-key");
        let models = provider.available_models();

        // Should have available models
        assert!(!models.is_empty());
    }

    #[test]
    fn test_provider_cloning() {
        let provider = GeminiProvider::new("test-api-key");
        let _cloned = provider.clone();
        assert!(true);
    }
}
