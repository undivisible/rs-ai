use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use rusty_ai::{AiResult, GenerateOptions, GenerateResult, Middleware, MiddlewareNext, Prompt};

/// A cached generate result together with its insertion timestamp.
struct CacheEntry {
    result: GenerateResult,
    inserted_at: Instant,
}

/// In-memory caching middleware for non-streaming generate calls.
///
/// Caches responses keyed on a hash of the prompt. Entries expire after
/// the configured TTL.  The cache is thread-safe via `Arc<Mutex<_>>`.
pub struct CacheMiddleware {
    cache: Arc<Mutex<HashMap<u64, CacheEntry>>>,
    ttl: Duration,
}

impl CacheMiddleware {
    /// Create a new `CacheMiddleware` with the specified time-to-live.
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl,
        }
    }

    /// Compute a deterministic hash for a prompt so it can be used as a cache key.
    fn hash_prompt(prompt: &Prompt) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Serialize the prompt to JSON for a stable, content-based hash.
        if let Ok(json) = serde_json::to_string(prompt) {
            json.hash(&mut hasher);
        }
        hasher.finish()
    }
}

#[async_trait]
impl Middleware for CacheMiddleware {
    async fn process(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
        next: MiddlewareNext<'_>,
    ) -> AiResult<GenerateResult> {
        let key = Self::hash_prompt(&prompt);

        // Check cache.
        {
            let cache = self.cache.lock().expect("cache lock poisoned");
            if let Some(entry) = cache.get(&key) {
                if entry.inserted_at.elapsed() < self.ttl {
                    tracing::debug!(cache_key = key, "cache hit");
                    return Ok(entry.result.clone());
                }
            }
        }

        tracing::debug!(cache_key = key, "cache miss");

        // Execute the downstream chain.
        let result = next.run(prompt, options).await?;

        // Store in cache.
        {
            let mut cache = self.cache.lock().expect("cache lock poisoned");

            // Evict expired entries opportunistically.
            cache.retain(|_, entry| entry.inserted_at.elapsed() < self.ttl);

            cache.insert(
                key,
                CacheEntry {
                    result: result.clone(),
                    inserted_at: Instant::now(),
                },
            );
        }

        Ok(result)
    }
}
