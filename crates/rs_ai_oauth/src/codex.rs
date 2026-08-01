use base64::engine::general_purpose::{STANDARD_NO_PAD, URL_SAFE_NO_PAD};
use base64::Engine;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const CHATGPT_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
pub const CHATGPT_CODEX_MODELS: &[&str] =
    &["gpt-5.5", "gpt-5.4", "gpt-5.4-mini", "gpt-5.3-codex-spark"];

#[derive(Debug, Clone)]
pub struct CodexToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Default)]
pub struct CodexResponse {
    pub text: String,
    pub tool_calls: Vec<CodexToolCall>,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Clone)]
pub struct ChatGptCodexClient {
    access_token: String,
    account_id: Option<String>,
    base_url: String,
    originator: String,
}

impl ChatGptCodexClient {
    pub fn new(access_token: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            account_id: None,
            base_url: CHATGPT_CODEX_BASE_URL.to_string(),
            originator: "rs_ai".to_string(),
        }
        .with_account_id_from_token()
    }

    pub fn with_account_id(mut self, account_id: impl Into<String>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_originator(mut self, originator: impl Into<String>) -> Self {
        self.originator = originator.into();
        self
    }

    pub fn account_id(&self) -> Option<&str> {
        self.account_id.as_deref()
    }

    pub async fn complete(
        &self,
        body: Value,
        session_id: Option<&str>,
    ) -> Result<CodexResponse, crate::OAuthError> {
        let url = codex_endpoint(&self.base_url);
        let response = reqwest::Client::new()
            .post(url)
            .headers(self.headers(session_id)?)
            .json(&body)
            .send()
            .await
            .map_err(|error| {
                crate::OAuthError::Network(format!("ChatGPT Codex request failed: {error}"))
            })?;

        let status = response.status();
        let text = response.text().await.map_err(|error| {
            crate::OAuthError::Network(format!("ChatGPT Codex response read failed: {error}"))
        })?;
        if !status.is_success() {
            return Err(crate::OAuthError::Auth(format!(
                "ChatGPT Codex request failed (HTTP {status}): {}",
                error_detail(&text)
            )));
        }

        parse_sse(&text)
    }

    fn with_account_id_from_token(mut self) -> Self {
        self.account_id = account_id_from_token(&self.access_token);
        self
    }

    fn headers(&self, session_id: Option<&str>) -> Result<HeaderMap, crate::OAuthError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::try_from(format!("Bearer {}", self.access_token)).map_err(|error| {
                crate::OAuthError::Auth(format!("invalid ChatGPT access token: {error}"))
            })?,
        );
        headers.insert(
            "originator",
            HeaderValue::try_from(self.originator.as_str()).map_err(|error| {
                crate::OAuthError::Auth(format!("invalid ChatGPT originator: {error}"))
            })?,
        );
        headers.insert(
            "user-agent",
            HeaderValue::try_from(format!("{}-codex", self.originator)).map_err(|error| {
                crate::OAuthError::Auth(format!("invalid ChatGPT user agent: {error}"))
            })?,
        );
        headers.insert(
            "openai-beta",
            HeaderValue::from_static("responses=experimental"),
        );
        headers.insert("accept", HeaderValue::from_static("text/event-stream"));
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        if let Some(account_id) = &self.account_id {
            headers.insert(
                "chatgpt-account-id",
                HeaderValue::try_from(account_id.as_str()).map_err(|error| {
                    crate::OAuthError::Auth(format!("invalid ChatGPT account ID: {error}"))
                })?,
            );
        }
        if let Some(session_id) = session_id {
            let value = HeaderValue::try_from(session_id).map_err(|error| {
                crate::OAuthError::Auth(format!("invalid ChatGPT session ID: {error}"))
            })?;
            headers.insert("session-id", value.clone());
            headers.insert("x-client-request-id", value);
        }
        Ok(headers)
    }
}

pub fn codex_request_body(
    model: &str,
    instructions: &str,
    input: Vec<Value>,
    tools: Vec<Value>,
    reasoning_effort: Option<&str>,
) -> Value {
    let mut body = json!({
        "model": model,
        "store": false,
        "stream": true,
        "instructions": instructions,
        "input": input,
        "text": {"verbosity": "low"},
        "include": ["reasoning.encrypted_content"],
        "tool_choice": "auto",
        "parallel_tool_calls": true,
    });
    if !tools.is_empty() {
        body["tools"] = Value::Array(tools);
    }
    if let Some(effort) = reasoning_effort {
        body["reasoning"] = json!({"effort": effort, "summary": "auto"});
    }
    body
}

fn codex_endpoint(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    if base.ends_with("/responses") {
        base.to_string()
    } else if base.ends_with("/codex") {
        format!("{base}/responses")
    } else {
        format!("{base}/codex/responses")
    }
}

fn account_id_from_token(token: &str) -> Option<String> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .or_else(|_| STANDARD_NO_PAD.decode(payload))
        .ok()?;
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    value
        .get("chatgpt_account_id")
        .or_else(|| {
            value
                .get("https://api.openai.com/auth")?
                .get("chatgpt_account_id")
        })
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn error_detail(text: &str) -> String {
    serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|value| {
            value
                .pointer("/error/message")
                .or_else(|| value.pointer("/error/error_description"))
                .or_else(|| value.get("message"))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| text.trim().to_string())
}

fn parse_sse(text: &str) -> Result<CodexResponse, crate::OAuthError> {
    let mut response = CodexResponse::default();
    let mut calls: BTreeMap<usize, CodexToolCall> = BTreeMap::new();

    for block in text.split("\n\n") {
        let data = block
            .lines()
            .filter_map(|line| line.strip_prefix("data:"))
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("\n");
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        let event: Value = serde_json::from_str(&data).map_err(|error| {
            crate::OAuthError::Network(format!("invalid ChatGPT Codex SSE event: {error}"))
        })?;
        let event_type = event
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        match event_type {
            "response.output_text.delta" => {
                response.text.push_str(
                    event
                        .get("delta")
                        .and_then(Value::as_str)
                        .unwrap_or_default(),
                );
            }
            "response.output_item.added" | "response.output_item.done" => {
                let item = event.get("item").unwrap_or(&Value::Null);
                if item.get("type").and_then(Value::as_str) == Some("function_call") {
                    let index = event
                        .get("output_index")
                        .and_then(Value::as_u64)
                        .unwrap_or(calls.len() as u64) as usize;
                    let call = calls.entry(index).or_insert_with(|| CodexToolCall {
                        id: String::new(),
                        name: String::new(),
                        arguments: String::new(),
                    });
                    if let Some(value) = item.get("call_id").and_then(Value::as_str) {
                        call.id = value.to_string();
                    }
                    if let Some(value) = item.get("name").and_then(Value::as_str) {
                        call.name = value.to_string();
                    }
                    if let Some(value) = item.get("arguments").and_then(Value::as_str) {
                        call.arguments = value.to_string();
                    }
                }
            }
            "response.function_call_arguments.delta" => {
                let index = event
                    .get("output_index")
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as usize;
                calls
                    .entry(index)
                    .or_insert_with(|| CodexToolCall {
                        id: event
                            .get("call_id")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        name: event
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        arguments: String::new(),
                    })
                    .arguments
                    .push_str(
                        event
                            .get("delta")
                            .and_then(Value::as_str)
                            .unwrap_or_default(),
                    );
            }
            "response.function_call_arguments.done" => {
                let index = event
                    .get("output_index")
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as usize;
                calls
                    .entry(index)
                    .or_insert_with(|| CodexToolCall {
                        id: event
                            .get("call_id")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        name: event
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        arguments: String::new(),
                    })
                    .arguments = event
                    .get("arguments")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
            }
            "response.completed" | "response.done" | "response.incomplete" => {
                if let Some(usage) = event.pointer("/response/usage") {
                    response.input_tokens = usage
                        .get("input_tokens")
                        .and_then(Value::as_u64)
                        .unwrap_or_default();
                    response.output_tokens = usage
                        .get("output_tokens")
                        .and_then(Value::as_u64)
                        .unwrap_or_default();
                }
            }
            "error" | "response.failed" => {
                return Err(crate::OAuthError::Auth(error_detail(&event.to_string())));
            }
            _ => {}
        }
    }

    response.tool_calls = calls.into_values().collect();
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_codex_models_are_listed() {
        assert_eq!(CHATGPT_CODEX_MODELS[0], "gpt-5.5");
        assert!(CHATGPT_CODEX_MODELS.contains(&"gpt-5.4-mini"));
    }

    #[test]
    fn codex_endpoint_normalizes_base_urls() {
        assert_eq!(
            codex_endpoint(CHATGPT_CODEX_BASE_URL),
            "https://chatgpt.com/backend-api/codex/responses"
        );
        assert_eq!(
            codex_endpoint("https://chatgpt.com/backend-api/codex/"),
            "https://chatgpt.com/backend-api/codex/responses"
        );
        assert_eq!(
            codex_endpoint("https://chatgpt.com/backend-api/codex/responses"),
            "https://chatgpt.com/backend-api/codex/responses"
        );
    }

    #[test]
    fn parses_text_and_tool_events() {
        let body = concat!(
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"hello\"}\n\n",
            "data: {\"type\":\"response.output_item.added\",\"output_index\":0,\"item\":{\"type\":\"function_call\",\"call_id\":\"call_1\",\"name\":\"shell\"}}\n\n",
            "data: {\"type\":\"response.function_call_arguments.done\",\"output_index\":0,\"arguments\":\"{}\"}\n\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"usage\":{\"input_tokens\":2,\"output_tokens\":3}}}\n\n",
        );
        let response = parse_sse(body).expect("parse");
        assert_eq!(response.text, "hello");
        assert_eq!(response.tool_calls[0].name, "shell");
        assert_eq!(response.input_tokens, 2);
        assert_eq!(response.output_tokens, 3);
    }
}
