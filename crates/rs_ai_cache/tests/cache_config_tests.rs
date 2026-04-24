use rs_ai_cache::{CacheConfig, CacheTTL};

#[test]
fn test_cache_config_default() {
    let config = CacheConfig::default();
    assert!(!config.enabled); // default() sets to false, new() sets to true
    assert_eq!(config.ttl, CacheTTL::FiveMinutes);
    assert!(!config.claude_ephemeral); // Default for bool is false
    assert!(config.gemini_cache_key.is_none());
    assert!(config.openai_cache_key.is_none());
    assert!(config.xai_conv_id.is_none());
    assert!(config.prompt_cache_key.is_none());
}

#[test]
fn test_cache_config_disabled() {
    let config = CacheConfig::new().disable();
    assert!(!config.enabled);
}

#[test]
fn test_cache_ttl_default() {
    let ttl = CacheTTL::default();
    assert_eq!(ttl, CacheTTL::FiveMinutes);
    assert_eq!(ttl.seconds(), Some(300));
}

#[test]
fn test_cache_ttl_custom() {
    let ttl = CacheTTL::Custom(7200);
    assert_eq!(ttl.seconds(), Some(7200));
    assert_eq!(ttl.as_anthropic_str(), None);
}

#[test]
fn test_cache_ttl_in_memory() {
    let ttl = CacheTTL::InMemory;
    assert_eq!(ttl.seconds(), None);
}

#[test]
fn test_cache_config_all_providers() {
    let config = CacheConfig::new()
        .with_gemini_cache_key("gemini-key-123")
        .with_openai_cache_key("openai-key-456")
        .with_xai_conv_id("conv-789")
        .with_prompt_cache_key("prompt-abc");

    assert_eq!(config.gemini_cache_key, Some("gemini-key-123".to_string()));
    assert_eq!(config.openai_cache_key, Some("openai-key-456".to_string()));
    assert_eq!(config.xai_conv_id, Some("conv-789".to_string()));
    assert_eq!(config.prompt_cache_key, Some("prompt-abc".to_string()));
}

#[test]
fn test_cache_config_enable_then_disable() {
    let config = CacheConfig::new().disable().enable();
    assert!(config.enabled);
}

#[test]
fn test_cache_config_ttl_changes() {
    let config = CacheConfig::new()
        .with_ttl(CacheTTL::OneHour)
        .with_ttl(CacheTTL::TwentyFourHours);

    assert_eq!(config.ttl, CacheTTL::TwentyFourHours);
    assert_eq!(config.ttl.seconds(), Some(86400));
}

#[test]
fn test_cache_config_ephemeral_mode() {
    let config = CacheConfig::new().enable_anthropic_ephemeral();
    assert!(config.claude_ephemeral);
    assert_eq!(config.ttl, CacheTTL::FiveMinutes);
}

#[test]
fn test_cache_config_persistent_mode() {
    let config = CacheConfig::new().enable_anthropic_persistent();
    assert!(!config.claude_ephemeral);
    assert_eq!(config.ttl, CacheTTL::OneHour);
}

#[test]
fn test_cache_config_mode_switching() {
    let config = CacheConfig::new()
        .enable_anthropic_ephemeral()
        .enable_anthropic_persistent();

    assert!(!config.claude_ephemeral);
    assert_eq!(config.ttl, CacheTTL::OneHour);
}

#[test]
fn test_cache_config_clone() {
    let config1 = CacheConfig::new()
        .with_xai_conv_id("conv-123")
        .with_ttl(CacheTTL::OneHour);
    let config2 = config1.clone();

    assert_eq!(config1.xai_conv_id, config2.xai_conv_id);
    assert_eq!(config1.ttl, config2.ttl);
}

#[test]
fn test_cache_config_partial_setup() {
    let config = CacheConfig::new().with_gemini_cache_key("key1");

    assert_eq!(config.gemini_cache_key, Some("key1".to_string()));
    assert!(config.openai_cache_key.is_none());
    assert!(config.xai_conv_id.is_none());
}

#[test]
fn test_cache_ttl_all_variants() {
    let variants = vec![
        (CacheTTL::FiveMinutes, Some(300)),
        (CacheTTL::OneHour, Some(3600)),
        (CacheTTL::TwentyFourHours, Some(86400)),
        (CacheTTL::InMemory, None),
        (CacheTTL::Custom(999), Some(999)),
    ];

    for (ttl, expected_seconds) in variants {
        assert_eq!(ttl.seconds(), expected_seconds);
    }
}

#[test]
fn test_cache_config_serialization() {
    let config = CacheConfig::new()
        .with_gemini_cache_key("key123")
        .with_xai_conv_id("conv456");

    let json = serde_json::to_string(&config).expect("should serialize");
    let parsed: CacheConfig = serde_json::from_str(&json).expect("should deserialize");

    assert_eq!(parsed.gemini_cache_key, config.gemini_cache_key);
    assert_eq!(parsed.xai_conv_id, config.xai_conv_id);
}

#[test]
fn test_default_enabled_helper() {
    let config = CacheConfig::default_enabled();
    assert!(config.enabled);
    assert_eq!(config.ttl, CacheTTL::FiveMinutes);
}
