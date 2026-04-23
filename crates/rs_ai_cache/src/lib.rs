//! Unified caching configuration for RAI language models.
//!
//! Provides a provider-agnostic caching interface that adapts to each provider's
//! caching mechanism:
//! - **Claude**: Explicit cache_control with ephemeral TTL
//! - **Gemini**: Cached content reuse with cache keys
//! - **OpenAI**: Automatic caching with optional routing hints
//! - **xAI**: Conversation routing and prompt cache keys
//!
//! # Example
//!
//! ```ignore
//! use rai_cache::CacheConfig;
//!
//! let cache = CacheConfig::default()
//!     .with_ttl(CacheTTL::FiveMinutes)
//!     .enable_anthropic_ephemeral();
//!
//! model.set_cache(cache);
//! ```

use serde::{Deserialize, Serialize};

/// Cache TTL configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheTTL {
    /// 5-minute ephemeral cache (Anthropic, xAI default)
    FiveMinutes,
    /// 1-hour persistent cache (Anthropic)
    OneHour,
    /// 24-hour persistent cache (OpenAI)
    TwentyFourHours,
    /// In-memory only (OpenAI default, ~5-10 min)
    InMemory,
    /// Custom duration in seconds
    Custom(u64),
}

impl CacheTTL {
    pub fn seconds(&self) -> Option<u64> {
        match self {
            CacheTTL::FiveMinutes => Some(300),
            CacheTTL::OneHour => Some(3600),
            CacheTTL::TwentyFourHours => Some(86400),
            CacheTTL::InMemory => None,
            CacheTTL::Custom(s) => Some(*s),
        }
    }

    pub fn as_anthropic_str(&self) -> Option<&'static str> {
        match self {
            CacheTTL::FiveMinutes => Some("ephemeral"),
            CacheTTL::OneHour => Some("persistent"),
            _ => None,
        }
    }
}

impl Default for CacheTTL {
    fn default() -> Self {
        CacheTTL::FiveMinutes
    }
}

/// Unified cache configuration for all providers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching for this request
    pub enabled: bool,

    /// Cache TTL/duration
    pub ttl: CacheTTL,

    // Provider-specific fields
    /// Claude: Use ephemeral (5-min) vs persistent (1-hour) cache
    pub claude_ephemeral: bool,

    /// Gemini: Optional cache key for explicit caching
    pub gemini_cache_key: Option<String>,

    /// OpenAI: Optional cache key for routing to same backend
    pub openai_cache_key: Option<String>,

    /// xAI: Conversation ID for server routing and caching
    pub xai_conv_id: Option<String>,

    /// xAI/OpenAI: Prompt cache key for cache reuse
    pub prompt_cache_key: Option<String>,
}

impl CacheConfig {
    /// Create a new cache config with caching enabled.
    pub fn new() -> Self {
        Self {
            enabled: true,
            ttl: CacheTTL::default(),
            claude_ephemeral: true,
            gemini_cache_key: None,
            openai_cache_key: None,
            xai_conv_id: None,
            prompt_cache_key: None,
        }
    }

    /// Enable caching
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Disable caching
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Set cache TTL
    pub fn with_ttl(mut self, ttl: CacheTTL) -> Self {
        self.ttl = ttl;
        self
    }

    /// Set Claude to use ephemeral (5-min) cache
    pub fn enable_anthropic_ephemeral(mut self) -> Self {
        self.claude_ephemeral = true;
        self.ttl = CacheTTL::FiveMinutes;
        self
    }

    /// Set Claude to use persistent (1-hour) cache
    pub fn enable_anthropic_persistent(mut self) -> Self {
        self.claude_ephemeral = false;
        self.ttl = CacheTTL::OneHour;
        self
    }

    /// Set Gemini cache key for explicit caching
    pub fn with_gemini_cache_key(mut self, key: impl Into<String>) -> Self {
        self.gemini_cache_key = Some(key.into());
        self
    }

    /// Set OpenAI cache routing key
    pub fn with_openai_cache_key(mut self, key: impl Into<String>) -> Self {
        self.openai_cache_key = Some(key.into());
        self
    }

    /// Set xAI conversation ID for routing
    pub fn with_xai_conv_id(mut self, conv_id: impl Into<String>) -> Self {
        self.xai_conv_id = Some(conv_id.into());
        self
    }

    /// Set prompt cache key (xAI, OpenAI)
    pub fn with_prompt_cache_key(mut self, key: impl Into<String>) -> Self {
        self.prompt_cache_key = Some(key.into());
        self
    }

    /// Enable caching with all defaults
    pub fn default_enabled() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_ttl_seconds() {
        assert_eq!(CacheTTL::FiveMinutes.seconds(), Some(300));
        assert_eq!(CacheTTL::OneHour.seconds(), Some(3600));
        assert_eq!(CacheTTL::InMemory.seconds(), None);
        assert_eq!(CacheTTL::Custom(123).seconds(), Some(123));
    }

    #[test]
    fn test_cache_config_fluent() {
        let config = CacheConfig::new()
            .with_ttl(CacheTTL::OneHour)
            .with_xai_conv_id("conv-123")
            .with_prompt_cache_key("pk-abc");

        assert!(config.enabled);
        assert_eq!(config.ttl, CacheTTL::OneHour);
        assert_eq!(config.xai_conv_id, Some("conv-123".to_string()));
        assert_eq!(config.prompt_cache_key, Some("pk-abc".to_string()));
    }

    #[test]
    fn test_anthropic_cache_type() {
        let ephemeral = CacheConfig::new().enable_anthropic_ephemeral();
        assert!(ephemeral.claude_ephemeral);
        assert_eq!(ephemeral.ttl.as_anthropic_str(), Some("ephemeral"));

        let persistent = CacheConfig::new().enable_anthropic_persistent();
        assert!(!persistent.claude_ephemeral);
        assert_eq!(persistent.ttl.as_anthropic_str(), Some("persistent"));
    }
}
