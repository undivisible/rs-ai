//! LangFuse HTTP client for emitting observability events.

use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use super::error::{LangfuseError, LangfuseResult};

/// LangFuse API endpoint for ingestion.
const LANGFUSE_API_URL: &str = "https://api.langfuse.com/api/public/ingestion";

/// Represents a single observability event to emit to LangFuse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangfuseEvent {
    /// Unique trace ID for grouping events.
    pub trace_id: String,

    /// Event name (e.g., "ai.generate", "ai.stream").
    pub name: String,

    /// Model identifier (e.g., "claude-sonnet-4-6").
    pub model: String,

    /// Provider identifier (e.g., "anthropic").
    pub provider: String,

    /// Number of prompt tokens (optional).
    pub prompt_tokens: Option<u64>,

    /// Number of completion tokens (optional).
    pub completion_tokens: Option<u64>,

    /// Total latency in milliseconds.
    pub latency_ms: u64,

    /// Cost in USD (optional).
    pub cost_usd: Option<f64>,

    /// Finish reason (optional).
    pub finish_reason: Option<String>,

    /// User ID for tracking (optional).
    pub user_id: Option<String>,

    /// Session ID for tracking (optional).
    pub session_id: Option<String>,

    /// Metadata as JSON (optional).
    pub metadata: Option<serde_json::Value>,
}

impl LangfuseEvent {
    /// Create a new LangFuse event with auto-generated trace ID.
    pub fn new(name: &str, model: &str, provider: &str) -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            model: model.to_string(),
            provider: provider.to_string(),
            prompt_tokens: None,
            completion_tokens: None,
            latency_ms: 0,
            cost_usd: None,
            finish_reason: None,
            user_id: None,
            session_id: None,
            metadata: None,
        }
    }

    /// Set the trace ID for this event.
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = trace_id;
        self
    }

    /// Set prompt tokens.
    pub fn with_prompt_tokens(mut self, tokens: u64) -> Self {
        self.prompt_tokens = Some(tokens);
        self
    }

    /// Set completion tokens.
    pub fn with_completion_tokens(mut self, tokens: u64) -> Self {
        self.completion_tokens = Some(tokens);
        self
    }

    /// Set latency in milliseconds.
    pub fn with_latency_ms(mut self, latency_ms: u64) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    /// Set cost in USD.
    pub fn with_cost_usd(mut self, cost: f64) -> Self {
        self.cost_usd = Some(cost);
        self
    }

    /// Set finish reason.
    pub fn with_finish_reason(mut self, reason: String) -> Self {
        self.finish_reason = Some(reason);
        self
    }

    /// Set user ID.
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set session ID.
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set metadata.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// HTTP client for communicating with LangFuse API.
pub struct LangfuseClient {
    http_client: reqwest::Client,
    public_key: String,
    secret_key: String,
}

impl LangfuseClient {
    /// Create a new LangFuse client with API credentials.
    ///
    /// # Arguments
    ///
    /// * `public_key` - Public API key from LangFuse dashboard
    /// * `secret_key` - Secret API key from LangFuse dashboard
    pub fn new(public_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            public_key: public_key.into(),
            secret_key: secret_key.into(),
        }
    }

    /// Create a LangFuse client from environment variables.
    ///
    /// Looks for `LANGFUSE_PUBLIC_KEY` and `LANGFUSE_SECRET_KEY`.
    pub fn from_env() -> LangfuseResult<Self> {
        let public_key = std::env::var("LANGFUSE_PUBLIC_KEY")
            .map_err(|_| LangfuseError::ConfigError("LANGFUSE_PUBLIC_KEY not set".to_string()))?;
        let secret_key = std::env::var("LANGFUSE_SECRET_KEY")
            .map_err(|_| LangfuseError::ConfigError("LANGFUSE_SECRET_KEY not set".to_string()))?;

        Ok(Self::new(public_key, secret_key))
    }

    /// Emit a trace event to LangFuse.
    ///
    /// This is an async operation that makes an HTTP request to the LangFuse API.
    /// Errors are logged but do not block or fail the caller's operation.
    pub async fn emit_trace(&self, event: LangfuseEvent) -> LangfuseResult<()> {
        let payload = self.build_payload(&event)?;
        let auth_header = self.build_auth_header();

        let response = self
            .http_client
            .post(LANGFUSE_API_URL)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LangfuseError::ApiError { status, message });
        }

        Ok(())
    }

    /// Emit a batch of trace events to LangFuse.
    ///
    /// Events are sent in a single HTTP request for efficiency.
    pub async fn emit_batch(&self, events: Vec<LangfuseEvent>) -> LangfuseResult<()> {
        if events.is_empty() {
            return Ok(());
        }

        let payloads: Result<Vec<_>, _> = events.iter().map(|e| self.build_payload(e)).collect();
        let payloads = payloads?;

        let batch_payload = json!({
            "batch": payloads,
        });

        let auth_header = self.build_auth_header();

        let response = self
            .http_client
            .post(LANGFUSE_API_URL)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(&batch_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LangfuseError::ApiError { status, message });
        }

        Ok(())
    }

    /// Build the Basic Auth header for LangFuse API.
    fn build_auth_header(&self) -> String {
        let credentials = format!("{}:{}", self.public_key, self.secret_key);
        let encoded = STANDARD.encode(credentials);
        format!("Basic {}", encoded)
    }

    /// Build the payload for a single trace event.
    fn build_payload(&self, event: &LangfuseEvent) -> LangfuseResult<serde_json::Value> {
        let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        let mut payload = json!({
            "id": Uuid::new_v4().to_string(),
            "type": "span",
            "timestamp": timestamp,
            "name": event.name,
            "traceId": event.trace_id,
            "metadata": {
                "model": event.model,
                "provider": event.provider,
                "latencyMs": event.latency_ms,
            }
        });

        // Add optional fields
        if let Some(pt) = event.prompt_tokens {
            payload["metadata"]["promptTokens"] = json!(pt);
        }
        if let Some(ct) = event.completion_tokens {
            payload["metadata"]["completionTokens"] = json!(ct);
        }
        if let Some(cost) = event.cost_usd {
            payload["metadata"]["costUsd"] = json!(cost);
        }
        if let Some(ref reason) = event.finish_reason {
            payload["metadata"]["finishReason"] = json!(reason);
        }
        if let Some(ref user_id) = event.user_id {
            payload["userId"] = json!(user_id);
        }
        if let Some(ref session_id) = event.session_id {
            payload["sessionId"] = json!(session_id);
        }
        if let Some(ref metadata) = event.metadata {
            payload["metadata"]["custom"] = metadata.clone();
        }

        Ok(payload)
    }
}

/// Shared reference to a LangFuse client.
pub type LangfuseClientRef = Arc<LangfuseClient>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_langfuse_event_builder() {
        let event = LangfuseEvent::new("ai.generate", "claude-3-5-sonnet", "anthropic")
            .with_prompt_tokens(100)
            .with_completion_tokens(50)
            .with_latency_ms(1500)
            .with_cost_usd(0.001);

        assert_eq!(event.name, "ai.generate");
        assert_eq!(event.model, "claude-3-5-sonnet");
        assert_eq!(event.provider, "anthropic");
        assert_eq!(event.prompt_tokens, Some(100));
        assert_eq!(event.completion_tokens, Some(50));
        assert_eq!(event.latency_ms, 1500);
        assert_eq!(event.cost_usd, Some(0.001));
    }

    #[test]
    fn test_auth_header_encoding() {
        let client = LangfuseClient::new("pub_key", "secret_key");
        let header = client.build_auth_header();
        assert!(header.starts_with("Basic "));
        // Decode and verify
        let encoded = header.strip_prefix("Basic ").unwrap();
        let decoded = STANDARD.decode(encoded).unwrap();
        assert_eq!(decoded, b"pub_key:secret_key");
    }

    #[test]
    fn test_build_payload() {
        let client = LangfuseClient::new("pub_key", "secret_key");
        let event = LangfuseEvent::new("ai.generate", "gpt-4", "openai")
            .with_prompt_tokens(100)
            .with_completion_tokens(50)
            .with_latency_ms(1000)
            .with_finish_reason("stop".to_string());

        let payload = client.build_payload(&event).unwrap();
        assert_eq!(payload["type"], "span");
        assert_eq!(payload["name"], "ai.generate");
        assert_eq!(payload["metadata"]["model"], "gpt-4");
        assert_eq!(payload["metadata"]["provider"], "openai");
        assert_eq!(payload["metadata"]["promptTokens"], 100);
        assert_eq!(payload["metadata"]["completionTokens"], 50);
        assert_eq!(payload["metadata"]["latencyMs"], 1000);
        assert_eq!(payload["metadata"]["finishReason"], "stop");
    }
}
