use crate::{
    AiError, AiResult, GenerateOptions, GenerateResult, Middleware, MiddlewareNext, Prompt,
};
use async_trait::async_trait;

/// Configuration for retry behaviour.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (not counting the initial request).
    pub max_retries: u32,
    /// Initial delay in milliseconds before the first retry.
    pub initial_delay_ms: u64,
    /// Multiplier applied to the delay after each retry.
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Middleware that retries failed requests with exponential backoff.
///
/// Only errors deemed "retryable" (rate limits, timeouts, transport failures)
/// trigger a retry. All other errors are propagated immediately.
pub struct RetryMiddleware {
    config: RetryConfig,
}

impl RetryMiddleware {
    /// Create a new `RetryMiddleware` with the given configuration.
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Returns `true` for error variants that are safe to retry.
    pub fn default_retryable(error: &AiError) -> bool {
        matches!(
            error,
            AiError::RateLimit { .. } | AiError::Timeout | AiError::Transport { .. }
        )
    }
}

#[async_trait]
impl Middleware for RetryMiddleware {
    async fn process(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
        next: MiddlewareNext<'_>,
    ) -> AiResult<GenerateResult> {
        let mut delay_ms = self.config.initial_delay_ms;

        // Save references so we can rebuild MiddlewareNext after consumption.
        let remaining_middlewares = next.middlewares;
        let model = next.model;

        let mut last_error: Option<AiError> = None;

        for attempt in 1..=(self.config.max_retries + 1) {
            if attempt > 1 {
                tracing::info!(attempt, delay_ms, "retrying request");
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                delay_ms = (delay_ms as f64 * self.config.backoff_multiplier) as u64;
            }

            let next_handle = MiddlewareNext {
                middlewares: remaining_middlewares,
                model,
            };

            match next_handle.run(prompt.clone(), options.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if !Self::default_retryable(&e) {
                        return Err(e);
                    }
                    tracing::warn!(
                        attempt,
                        max_retries = self.config.max_retries,
                        error = %e,
                        "retryable error encountered"
                    );
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AiError::Transport {
            message:
                "Retry loop exhausted without capturing an error (this is a bug in RetryMiddleware)"
                    .to_string(),
            source: None,
        }))
    }
}
