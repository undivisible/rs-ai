//! Integration tests for xAI cache configuration and conversation routing.

use rs_ai_cache::CacheConfig;
use rs_ai_xai::XaiProvider;

#[test]
fn test_model_with_conv_id() {
    let provider = XaiProvider::new("test-api-key");
    let model = provider.grok_4_20_reasoning().with_conv_id("conv-123-abc");

    let cache_config = model.cache_config();
    assert!(cache_config.is_some());
    let config = cache_config.unwrap();
    assert_eq!(config.xai_conv_id, Some("conv-123-abc".to_string()));
}

#[test]
fn test_model_with_prompt_cache_key() {
    let provider = XaiProvider::new("test-api-key");
    let model = provider.grok_4().with_prompt_cache_key("pk-xyz-789");

    let cache_config = model.cache_config();
    assert!(cache_config.is_some());
    let config = cache_config.unwrap();
    assert_eq!(config.prompt_cache_key, Some("pk-xyz-789".to_string()));
}

#[test]
fn test_model_with_both_conv_id_and_cache_key() {
    let provider = XaiProvider::new("test-api-key");
    let model = provider
        .grok_4()
        .with_conv_id("conv-456")
        .with_prompt_cache_key("pk-abc-123");

    let cache_config = model.cache_config();
    assert!(cache_config.is_some());
    let config = cache_config.unwrap();
    assert_eq!(config.xai_conv_id, Some("conv-456".to_string()));
    assert_eq!(config.prompt_cache_key, Some("pk-abc-123".to_string()));
}

#[test]
fn test_conversation_id_persistence() {
    let provider = XaiProvider::new("test-api-key");

    // First request with conversation ID
    let model1 = provider
        .grok_4_20_reasoning()
        .with_conv_id("shared-conv-id");

    let cache_config1 = model1.cache_config().unwrap();
    assert_eq!(
        cache_config1.xai_conv_id,
        Some("shared-conv-id".to_string())
    );

    // Second request with same conversation ID
    let model2 = provider.grok_4().with_conv_id("shared-conv-id");

    let cache_config2 = model2.cache_config().unwrap();
    assert_eq!(
        cache_config2.xai_conv_id,
        Some("shared-conv-id".to_string())
    );

    // Both should have same conversation ID
    assert_eq!(cache_config1.xai_conv_id, cache_config2.xai_conv_id);
}

#[test]
fn test_set_cache_method() {
    let provider = XaiProvider::new("test-api-key");
    let mut model = provider.grok_4();

    let cache_config = CacheConfig::new()
        .with_xai_conv_id("conv-789")
        .with_prompt_cache_key("pk-def-456");

    model.set_cache(cache_config);

    let retrieved_config = model.cache_config().unwrap();
    assert_eq!(retrieved_config.xai_conv_id, Some("conv-789".to_string()));
    assert_eq!(
        retrieved_config.prompt_cache_key,
        Some("pk-def-456".to_string())
    );
}

#[test]
fn test_provider_model_with_cache() {
    let provider = XaiProvider::new("test-api-key");

    let cache_config = CacheConfig::new()
        .with_xai_conv_id("conv-via-provider")
        .with_prompt_cache_key("pk-via-provider");

    let model = provider.model_with_cache("grok-4", cache_config);

    let retrieved_config = model.cache_config().unwrap();
    assert_eq!(
        retrieved_config.xai_conv_id,
        Some("conv-via-provider".to_string())
    );
    assert_eq!(
        retrieved_config.prompt_cache_key,
        Some("pk-via-provider".to_string())
    );
}

#[test]
fn test_provider_grok_4_20_with_cache() {
    let provider = XaiProvider::new("test-api-key");

    let cache_config = CacheConfig::new().with_xai_conv_id("conv-grok-420");

    let model = provider.grok_4_20_reasoning_with_cache(cache_config);

    let retrieved_config = model.cache_config().unwrap();
    assert_eq!(
        retrieved_config.xai_conv_id,
        Some("conv-grok-420".to_string())
    );
}

#[test]
fn test_provider_grok_4_with_cache() {
    let provider = XaiProvider::new("test-api-key");

    let cache_config = CacheConfig::new().with_xai_conv_id("conv-grok-4");

    let model = provider.grok_4_with_cache(cache_config);

    let retrieved_config = model.cache_config().unwrap();
    assert_eq!(
        retrieved_config.xai_conv_id,
        Some("conv-grok-4".to_string())
    );
}

#[test]
fn test_fluent_chain_conversions() {
    let provider = XaiProvider::new("test-api-key");
    let model = provider
        .grok_4()
        .with_conv_id("chain-test")
        .with_prompt_cache_key("chain-key");

    let config = model.cache_config().unwrap();
    assert_eq!(config.xai_conv_id, Some("chain-test".to_string()));
    assert_eq!(config.prompt_cache_key, Some("chain-key".to_string()));
}

#[test]
fn test_model_without_cache_config() {
    let provider = XaiProvider::new("test-api-key");
    let model = provider.grok_4();

    let cache_config = model.cache_config();
    assert!(cache_config.is_none());
}

#[test]
fn test_cache_config_default() {
    let cache = CacheConfig::default();
    assert!(!cache.enabled);
    assert_eq!(cache.xai_conv_id, None);
    assert_eq!(cache.prompt_cache_key, None);
}

#[test]
fn test_cache_config_new() {
    let cache = CacheConfig::new();
    assert!(cache.enabled);
    assert_eq!(cache.xai_conv_id, None);
    assert_eq!(cache.prompt_cache_key, None);
}
