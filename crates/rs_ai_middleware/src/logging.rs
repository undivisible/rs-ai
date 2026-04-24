use std::time::Instant;

use async_trait::async_trait;
use rs_ai_traits::{AiResult, GenerateOptions, GenerateResult, Middleware, MiddlewareNext, Prompt};

/// Middleware that logs request and response details using the `tracing` crate.
pub struct LoggingMiddleware {
    level: tracing::Level,
}

impl LoggingMiddleware {
    /// Create a new `LoggingMiddleware` that logs at `INFO` level.
    pub fn new() -> Self {
        Self {
            level: tracing::Level::INFO,
        }
    }

    /// Create a new `LoggingMiddleware` that logs at the specified level.
    pub fn with_level(level: tracing::Level) -> Self {
        Self { level }
    }

    /// Produce a short human-readable summary of the prompt.
    fn summarize_prompt(prompt: &Prompt) -> String {
        match prompt {
            Prompt::Text(t) => {
                let preview: String = t.chars().take(80).collect();
                if t.len() > 80 {
                    format!("\"{}...\" ({} chars)", preview, t.len())
                } else {
                    format!("\"{}\"", preview)
                }
            }
            Prompt::Messages(msgs) => {
                format!("{} message(s)", msgs.len())
            }
        }
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn process(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
        next: MiddlewareNext<'_>,
    ) -> AiResult<GenerateResult> {
        let summary = Self::summarize_prompt(&prompt);
        let temp = options.temperature;
        let max_tokens = options.max_tokens;

        match self.level {
            tracing::Level::TRACE => {
                tracing::trace!(
                    prompt = %summary,
                    temperature = ?temp,
                    max_tokens = ?max_tokens,
                    "generate request"
                );
            }
            tracing::Level::DEBUG => {
                tracing::debug!(
                    prompt = %summary,
                    temperature = ?temp,
                    max_tokens = ?max_tokens,
                    "generate request"
                );
            }
            tracing::Level::WARN => {
                tracing::warn!(
                    prompt = %summary,
                    temperature = ?temp,
                    max_tokens = ?max_tokens,
                    "generate request"
                );
            }
            tracing::Level::ERROR => {
                tracing::error!(
                    prompt = %summary,
                    temperature = ?temp,
                    max_tokens = ?max_tokens,
                    "generate request"
                );
            }
            _ => {
                tracing::info!(
                    prompt = %summary,
                    temperature = ?temp,
                    max_tokens = ?max_tokens,
                    "generate request"
                );
            }
        }

        let start = Instant::now();
        let result = next.run(prompt, options).await;
        let elapsed = start.elapsed();

        match &result {
            Ok(res) => {
                let prompt_tokens = res.usage.prompt_tokens;
                let completion_tokens = res.usage.completion_tokens;
                let finish_reason = &res.finish_reason;
                let latency_ms = elapsed.as_millis() as u64;

                match self.level {
                    tracing::Level::TRACE => {
                        tracing::trace!(latency_ms, prompt_tokens = ?prompt_tokens, completion_tokens = ?completion_tokens, finish_reason = ?finish_reason, "generate response")
                    }
                    tracing::Level::DEBUG => {
                        tracing::debug!(latency_ms, prompt_tokens = ?prompt_tokens, completion_tokens = ?completion_tokens, finish_reason = ?finish_reason, "generate response")
                    }
                    tracing::Level::WARN => {
                        tracing::warn!(latency_ms, prompt_tokens = ?prompt_tokens, completion_tokens = ?completion_tokens, finish_reason = ?finish_reason, "generate response")
                    }
                    tracing::Level::ERROR => {
                        tracing::error!(latency_ms, prompt_tokens = ?prompt_tokens, completion_tokens = ?completion_tokens, finish_reason = ?finish_reason, "generate response")
                    }
                    _ => {
                        tracing::info!(latency_ms, prompt_tokens = ?prompt_tokens, completion_tokens = ?completion_tokens, finish_reason = ?finish_reason, "generate response")
                    }
                }
            }
            Err(e) => {
                let latency_ms = elapsed.as_millis() as u64;
                // Use ?e (Debug) to preserve the full error source chain.
                match self.level {
                    tracing::Level::TRACE => {
                        tracing::trace!(latency_ms, error = ?e, "generate failed")
                    }
                    tracing::Level::DEBUG => {
                        tracing::debug!(latency_ms, error = ?e, "generate failed")
                    }
                    tracing::Level::WARN => {
                        tracing::warn!(latency_ms, error = ?e, "generate failed")
                    }
                    _ => tracing::error!(latency_ms, error = ?e, "generate failed"),
                }
            }
        }

        result
    }
}
