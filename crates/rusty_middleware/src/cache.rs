use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
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

    /// Compute a deterministic cache key from the prompt and generation options.
    ///
    /// Both the prompt content and all generation-affecting options are included
    /// so that requests with the same prompt but different parameters (temperature,
    /// tools, output schema, etc.) are treated as distinct cache entries.
    /// The request metadata (which contains a per-request UUID) is excluded.
    fn cache_key(prompt: &Prompt, options: &GenerateOptions) -> u64 {
        let mut hasher = DefaultHasher::new();

        if let Ok(json) = serde_json::to_string(prompt) {
            json.hash(&mut hasher);
        }

        // Numeric options — hash the bit pattern to keep f64 deterministic.
        options.temperature.map(f64::to_bits).hash(&mut hasher);
        options.max_tokens.hash(&mut hasher);
        options.top_p.map(f64::to_bits).hash(&mut hasher);
        options.top_k.hash(&mut hasher);
        options.stop_sequences.hash(&mut hasher);
        options.frequency_penalty.map(f64::to_bits).hash(&mut hasher);
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
        let key = Self::cache_key(&prompt, &options);

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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rusty_ai::{GenerateOptions, Prompt};
    use rusty_ai::tool::ToolDefinition;
    use rusty_testing::{MockLanguageModel, MockResponse};

    use crate::chain::MiddlewareChain;

    use super::CacheMiddleware;

    fn text_response(s: &str) -> MockResponse {
        MockResponse::Text(s.to_owned())
    }

    #[tokio::test]
    async fn same_prompt_and_options_hits_cache() {
        let model = MockLanguageModel::new("test")
            .with_response(text_response("response-1"));

        let chain = MiddlewareChain::new(model)
            .with(CacheMiddleware::new(Duration::from_secs(60)));

        let prompt = Prompt::from("hello");
        let opts = GenerateOptions::default();

        let r1 = chain.generate(prompt.clone(), opts.clone()).await.unwrap();
        let r2 = chain.generate(prompt.clone(), opts.clone()).await.unwrap();

        // Both should return the same cached text; the model was only called once.
        assert_eq!(r1.text, r2.text);
        assert_eq!(r1.text.as_deref(), Some("response-1"));
    }

    #[tokio::test]
    async fn different_temperature_is_a_cache_miss() {
        let model = MockLanguageModel::new("test")
            .with_response(text_response("response-cold"))
            .with_response(text_response("response-hot"));

        let chain = MiddlewareChain::new(model)
            .with(CacheMiddleware::new(Duration::from_secs(60)));

        let prompt = Prompt::from("same prompt");
        let cold = chain.generate(prompt.clone(), GenerateOptions::default().with_temperature(0.0)).await.unwrap();
        let hot = chain.generate(prompt.clone(), GenerateOptions::default().with_temperature(1.0)).await.unwrap();

        assert_eq!(cold.text.as_deref(), Some("response-cold"));
        assert_eq!(hot.text.as_deref(), Some("response-hot"));
    }

    #[tokio::test]
    async fn different_tools_is_a_cache_miss() {
        let model = MockLanguageModel::new("test")
            .with_response(text_response("no-tools"))
            .with_response(text_response("with-tools"));

        let chain = MiddlewareChain::new(model)
            .with(CacheMiddleware::new(Duration::from_secs(60)));

        let prompt = Prompt::from("same prompt");

        let r_plain = chain.generate(prompt.clone(), GenerateOptions::default()).await.unwrap();
        let r_tools = chain.generate(
            prompt.clone(),
            GenerateOptions::default().with_tools(vec![ToolDefinition {
                name: "search".into(),
                description: "search the web".into(),
                parameters: serde_json::json!({}),
            }]),
        ).await.unwrap();

        assert_eq!(r_plain.text.as_deref(), Some("no-tools"));
        assert_eq!(r_tools.text.as_deref(), Some("with-tools"));
    }

    #[tokio::test]
    async fn expired_entry_is_not_returned() {
        let model = MockLanguageModel::new("test")
            .with_response(text_response("first"))
            .with_response(text_response("second"));

        // TTL of 1ms so the entry expires immediately.
        let chain = MiddlewareChain::new(model)
            .with(CacheMiddleware::new(Duration::from_millis(1)));

        let prompt = Prompt::from("hello");
        let opts = GenerateOptions::default();

        let r1 = chain.generate(prompt.clone(), opts.clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(5)).await;
        let r2 = chain.generate(prompt.clone(), opts.clone()).await.unwrap();

        assert_eq!(r1.text.as_deref(), Some("first"));
        assert_eq!(r2.text.as_deref(), Some("second"), "expired entry should trigger a fresh model call");
    }
}
