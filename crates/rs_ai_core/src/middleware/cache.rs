use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::{AiResult, GenerateOptions, GenerateResult, Middleware, MiddlewareNext, Prompt};
use async_trait::async_trait;

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

    /// Compute a deterministic cache key from the prompt and generation options.
    ///
    /// Returns `None` if the prompt cannot be serialized, in which case the
    /// caller should bypass the cache entirely to avoid hash collisions.
    ///
    /// Both the prompt content and all generation-affecting options are included
    /// so that requests with the same prompt but different parameters (temperature,
    /// tools, output schema, etc.) are treated as distinct cache entries.
    /// The request metadata (which contains a per-request UUID) is excluded.
    fn cache_key(prompt: &Prompt, options: &GenerateOptions) -> Option<u64> {
        let mut hasher = DefaultHasher::new();

        let prompt_json = serde_json::to_string(prompt)
            .map_err(|e| tracing::error!(error = %e, "Failed to serialize prompt for cache key; bypassing cache"))
            .ok()?;
        prompt_json.hash(&mut hasher);

        // Numeric options — hash the bit pattern to keep f64 deterministic.
        options.temperature.map(f64::to_bits).hash(&mut hasher);
        options.max_tokens.hash(&mut hasher);
        options.top_p.map(f64::to_bits).hash(&mut hasher);
        options.top_k.hash(&mut hasher);
        options.stop_sequences.hash(&mut hasher);
        options
            .frequency_penalty
            .map(f64::to_bits)
            .hash(&mut hasher);
        options.presence_penalty.map(f64::to_bits).hash(&mut hasher);
        options.seed.hash(&mut hasher);

        // Complex types — serialize to JSON for a stable, content-based hash.
        if let Ok(json) = serde_json::to_string(&options.tools) {
            json.hash(&mut hasher);
        }
        if let Ok(json) = serde_json::to_string(&options.tool_choice) {
            json.hash(&mut hasher);
        }
        if let Ok(json) = serde_json::to_string(&options.output_schema) {
            json.hash(&mut hasher);
        }

        // Enum options without Serialize — use Debug, which is stable for owned enums.
        format!("{:?}", options.thinking).hash(&mut hasher);
        format!("{:?}", options.reasoning_effort).hash(&mut hasher);

        Some(hasher.finish())
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
        let Some(key) = Self::cache_key(&prompt, &options) else {
            // Prompt could not be serialized; bypass cache to avoid collisions.
            return next.run(prompt, options).await;
        };

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
