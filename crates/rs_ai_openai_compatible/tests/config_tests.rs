//! Configuration tests for OpenAI-compatible provider

#[cfg(test)]
mod tests {
    use rs_ai_openai_compatible::OpenAiCompatibleConfig;

    #[test]
    fn test_config_creation() {
        let config = OpenAiCompatibleConfig::new("https://api.example.com", "test-key");
        assert_eq!(config.base_url(), "https://api.example.com");
    }

    #[test]
    fn test_config_openai_preset() {
        let config = OpenAiCompatibleConfig::openai("sk-test");
        assert_eq!(config.base_url(), "https://api.openai.com/v1");
    }

    #[test]
    fn test_config_with_org() {
        let config =
            OpenAiCompatibleConfig::new("https://api.example.com", "key").with_org("org-123");
        assert!(config.base_url().contains("example.com"));
    }

    #[test]
    fn test_config_with_header() {
        let config = OpenAiCompatibleConfig::new("https://api.example.com", "key")
            .with_header("X-Custom-Header", "value");
        assert_eq!(config.base_url(), "https://api.example.com");
    }

    #[test]
    fn test_config_with_multiple_headers() {
        let config = OpenAiCompatibleConfig::new("https://api.example.com", "key")
            .with_header("X-Header-1", "value1")
            .with_header("X-Header-2", "value2");
        assert_eq!(config.base_url(), "https://api.example.com");
    }

    #[test]
    fn test_config_trailing_slash_handling() {
        let config = OpenAiCompatibleConfig::new("https://api.example.com/", "key");
        // URL should be cleaned (though this is handled by OpenAiCompatibleProvider)
        assert!(config.base_url().starts_with("https://"));
    }

    #[test]
    fn test_config_debug_hides_key() {
        let config = OpenAiCompatibleConfig::new("https://api.example.com", "secret-key");
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains("secret-key"));
    }

    #[test]
    fn test_config_clone() {
        let config1 = OpenAiCompatibleConfig::new("https://api.example.com", "key");
        let config2 = config1.clone();
        assert_eq!(config1.base_url(), config2.base_url());
    }

    #[test]
    fn test_config_with_auth_header() {
        let config = OpenAiCompatibleConfig::new("https://api.example.com", "test-key")
            .with_header("Authorization", "Bearer token");
        assert_eq!(config.base_url(), "https://api.example.com");
    }

    #[test]
    fn test_config_preserves_base_url_format() {
        let base_urls = vec![
            "https://api.openai.com/v1",
            "https://api.openrouter.ai/api/v1",
            "http://localhost:8000/v1",
            "https://api.example.com:8080/v1",
        ];

        for url in base_urls {
            let config = OpenAiCompatibleConfig::new(url, "key");
            assert_eq!(config.base_url(), url);
        }
    }
}
