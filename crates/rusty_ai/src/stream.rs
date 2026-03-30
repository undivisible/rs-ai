use std::pin::Pin;

use futures::stream::{self, Stream, StreamExt};

use crate::error::{AiError, AiResult};
use crate::structured::GenerateResult;
use crate::tool::ToolCallRequest;
use crate::types::{FinishReason, ResponseMetadata};
use crate::usage::Usage;

/// Events emitted by a streaming response.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    MessageStart { message_id: String },
    TextDelta { delta: String },
    ToolCallStart { call_id: String, tool_name: String },
    ToolCallDelta { call_id: String, delta: String },
    ToolCallEnd { call_id: String, arguments: serde_json::Value },
    ToolResult { call_id: String, content: String, is_error: bool },
    ObjectDelta { delta: serde_json::Value },
    UsageDelta { usage: Usage },
    Warning { message: String },
    MessageEnd { finish_reason: FinishReason, usage: Option<Usage> },
    Error { error: String },
}

/// A boxed, pinned, sendable stream of `StreamEvent` results.
pub type AiStream = Pin<Box<dyn Stream<Item = Result<StreamEvent, AiError>> + Send>>;

/// Collects a full `AiStream` into a single `GenerateResult`.
pub struct StreamCollector;

impl StreamCollector {
    pub async fn collect(mut stream: AiStream) -> AiResult<GenerateResult> {
        let mut text = String::new();
        let mut tool_calls: Vec<ToolCallRequest> = Vec::new();
        let mut finish_reason = FinishReason::Unknown;
        let mut usage = Usage::default();

        // Track in-progress tool calls by call_id
        let mut pending_tool_calls: std::collections::HashMap<String, (String, String)> =
            std::collections::HashMap::new();

        while let Some(event) = stream.next().await {
            let event = event?;
            match event {
                StreamEvent::TextDelta { delta } => {
                    text.push_str(&delta);
                }
                StreamEvent::ToolCallStart {
                    call_id,
                    tool_name,
                } => {
                    pending_tool_calls.insert(call_id, (tool_name, String::new()));
                }
                StreamEvent::ToolCallDelta { call_id, delta } => {
                    if let Some((_name, args)) = pending_tool_calls.get_mut(&call_id) {
                        args.push_str(&delta);
                    }
                }
                StreamEvent::ToolCallEnd { call_id, arguments } => {
                    if let Some((name, _partial_args)) = pending_tool_calls.remove(&call_id) {
                        tool_calls.push(ToolCallRequest {
                            id: call_id,
                            name,
                            arguments,
                        });
                    }
                }
                StreamEvent::UsageDelta { usage: u } => {
                    usage.merge(&u);
                }
                StreamEvent::MessageEnd {
                    finish_reason: fr,
                    usage: u,
                } => {
                    finish_reason = fr;
                    if let Some(u) = u {
                        usage.merge(&u);
                    }
                }
                StreamEvent::Error { error } => {
                    return Err(AiError::StreamError { message: error });
                }
                _ => {}
            }
        }

        Ok(GenerateResult {
            text: if text.is_empty() { None } else { Some(text) },
            tool_calls,
            finish_reason,
            usage,
            metadata: ResponseMetadata::default(),
        })
    }
}

/// Produces a synthetic stream from a complete text response.
///
/// Useful for providers that don't support real streaming.
pub struct SyntheticStreamer;

impl SyntheticStreamer {
    pub fn stream(text: String, chunk_size: usize) -> AiStream {
        let chunk_size = chunk_size.max(1);
        let chunks: Vec<Result<StreamEvent, AiError>> = {
            let mut v = Vec::new();
            v.push(Ok(StreamEvent::MessageStart {
                message_id: uuid::Uuid::new_v4().to_string(),
            }));

            let mut pos = 0;
            while pos < text.len() {
                let end = (pos + chunk_size).min(text.len());
                // Ensure we don't split in the middle of a multi-byte char.
                let end = if end < text.len() {
                    let mut e = end;
                    while !text.is_char_boundary(e) && e > pos {
                        e -= 1;
                    }
                    e
                } else {
                    end
                };
                if end == pos {
                    break;
                }
                v.push(Ok(StreamEvent::TextDelta {
                    delta: text[pos..end].to_owned(),
                }));
                pos = end;
            }

            v.push(Ok(StreamEvent::MessageEnd {
                finish_reason: FinishReason::Stop,
                usage: None,
            }));
            v
        };

        Box::pin(stream::iter(chunks))
    }
}
