use std::collections::HashMap;

use futures::stream::{self, StreamExt};
use reqwest::Response;
use rai_ai::error::AiError;
use rai_ai::stream::{AiStream, StreamEvent as RustyStreamEvent};
use rai_ai::Usage;

use crate::api_types::{ContentBlock, DeltaBlock, StreamEvent as AnthropicEvent};
use crate::convert::map_stop_reason;

/// State tracked while parsing a streaming response from the Anthropic API.
struct StreamState {
    message_id: String,
    /// Maps content-block index to tool-call metadata (id, name, accumulated
    /// JSON fragments).
    active_tool_calls: HashMap<usize, ToolCallState>,
    input_tokens: u64,
    output_tokens: u64,
}

struct ToolCallState {
    id: String,
    #[allow(dead_code)]
    name: String,
    json_buf: String,
}

impl StreamState {
    fn new() -> Self {
        Self {
            message_id: String::new(),
            active_tool_calls: HashMap::new(),
            input_tokens: 0,
            output_tokens: 0,
        }
    }
}

/// Parse an SSE response from the Anthropic Messages API into an `AiStream`.
pub(crate) fn parse_stream(response: Response) -> AiStream {
    let byte_stream = response.bytes_stream();

    // We'll buffer bytes and split by SSE boundary ("\n\n").
    let event_stream = futures::stream::unfold(
        (byte_stream, String::new()),
        |(mut byte_stream, mut buffer)| async move {
            loop {
                // Try to extract a complete SSE event from the buffer.
                if let Some(pos) = buffer.find("\n\n") {
                    let event_text = buffer[..pos].to_string();
                    buffer = buffer[pos + 2..].to_string();
                    let events = parse_sse_event(&event_text);
                    if !events.is_empty() {
                        return Some((events, (byte_stream, buffer)));
                    }
                    continue;
                }

                // Need more data.
                match byte_stream.next().await {
                    Some(Ok(chunk)) => {
                        let text = String::from_utf8_lossy(&chunk);
                        buffer.push_str(&text);
                    }
                    Some(Err(e)) => {
                        let msg = e.to_string();
                        return Some((
                            vec![Err(AiError::Transport {
                                message: msg,
                                source: Some(Box::new(e)),
                            })],
                            (byte_stream, buffer),
                        ));
                    }
                    None => {
                        // End of stream. Try to parse any remaining data.
                        if !buffer.trim().is_empty() {
                            let events = parse_sse_event(&buffer);
                            buffer.clear();
                            if !events.is_empty() {
                                return Some((events, (byte_stream, buffer)));
                            }
                        }
                        return None;
                    }
                }
            }
        },
    )
    .flat_map(stream::iter);

    // Now map Anthropic events to rai_ai StreamEvents using stateful processing.
    let mapped = futures::stream::unfold(
        (Box::pin(event_stream), StreamState::new()),
        |(mut event_stream, mut state)| async move {
            loop {
                match event_stream.next().await {
                    Some(Ok(anthropic_event)) => {
                        let rai_events = map_event(anthropic_event, &mut state);
                        if !rai_events.is_empty() {
                            let items: Vec<Result<RustyStreamEvent, AiError>> =
                                rai_events.into_iter().map(Ok).collect();
                            return Some((stream::iter(items), (event_stream, state)));
                        }
                        // Event produced no output (e.g. Ping), continue.
                        continue;
                    }
                    Some(Err(e)) => {
                        return Some((stream::iter(vec![Err(e)]), (event_stream, state)));
                    }
                    None => return None,
                }
            }
        },
    )
    .flat_map(|items| items);

    Box::pin(mapped)
}

/// Parse a single SSE text block (lines between double-newlines) into zero or
/// more `AnthropicEvent` results.
fn parse_sse_event(raw: &str) -> Vec<Result<AnthropicEvent, AiError>> {
    let mut event_type: Option<&str> = None;
    let mut data_lines: Vec<&str> = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("event:") {
            event_type = Some(rest.trim());
        } else if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim());
        }
        // Ignore other SSE fields (id:, retry:, comments starting with :).
    }

    if data_lines.is_empty() {
        return Vec::new();
    }

    let data = data_lines.join("\n");
    let _ = event_type; // The type is embedded in the JSON payload.

    match serde_json::from_str::<AnthropicEvent>(&data) {
        Ok(event) => vec![Ok(event)],
        Err(e) => {
            tracing::error!(data = %data, error = %e, "Failed to parse Anthropic SSE event; terminating stream");
            vec![Err(AiError::StreamError {
                message: format!("Unparseable SSE event from Anthropic: {e}"),
            })]
        }
    }
}

/// Map a single `AnthropicEvent` into zero or more `RustyStreamEvent` values.
fn map_event(event: AnthropicEvent, state: &mut StreamState) -> Vec<RustyStreamEvent> {
    match event {
        AnthropicEvent::MessageStart { message } => {
            state.message_id = message.id.clone();
            state.input_tokens = message.usage.input_tokens;
            state.output_tokens = message.usage.output_tokens;
            vec![
                RustyStreamEvent::MessageStart {
                    message_id: message.id,
                },
                RustyStreamEvent::UsageDelta {
                    usage: Usage {
                        prompt_tokens: Some(message.usage.input_tokens),
                        completion_tokens: Some(message.usage.output_tokens),
                        total_tokens: Some(
                            message.usage.input_tokens + message.usage.output_tokens,
                        ),
                    },
                },
            ]
        }

        AnthropicEvent::ContentBlockStart {
            index,
            content_block,
        } => match content_block {
            ContentBlock::Text { text, .. } => {
                if text.is_empty() {
                    Vec::new()
                } else {
                    vec![RustyStreamEvent::TextDelta { delta: text }]
                }
            }
            ContentBlock::ToolUse { id, name, .. } => {
                state.active_tool_calls.insert(
                    index,
                    ToolCallState {
                        id: id.clone(),
                        name: name.clone(),
                        json_buf: String::new(),
                    },
                );
                vec![RustyStreamEvent::ToolCallStart {
                    call_id: id,
                    tool_name: name,
                }]
            }
            _ => Vec::new(),
        },

        AnthropicEvent::ContentBlockDelta { index, delta } => match delta {
            DeltaBlock::Text { text } => {
                vec![RustyStreamEvent::TextDelta { delta: text }]
            }
            DeltaBlock::InputJson { partial_json } => {
                if let Some(tc) = state.active_tool_calls.get_mut(&index) {
                    tc.json_buf.push_str(&partial_json);
                    vec![RustyStreamEvent::ToolCallDelta {
                        call_id: tc.id.clone(),
                        delta: partial_json,
                    }]
                } else {
                    Vec::new()
                }
            }
            DeltaBlock::Thinking { thinking } => {
                vec![RustyStreamEvent::ThinkingDelta { delta: thinking }]
            }
            DeltaBlock::Signature { .. } => Vec::new(),
        },

        AnthropicEvent::ContentBlockStop { index } => {
            if let Some(tc) = state.active_tool_calls.remove(&index) {
                match serde_json::from_str(&tc.json_buf) {
                    Ok(arguments) => vec![RustyStreamEvent::ToolCallEnd {
                        call_id: tc.id,
                        arguments,
                    }],
                    Err(e) => {
                        tracing::error!(
                            call_id = %tc.id,
                            tool_name = %tc.name,
                            error = %e,
                            "Malformed tool call JSON buffer; cannot reconstruct arguments"
                        );
                        vec![RustyStreamEvent::Error {
                            error: format!(
                                "Malformed tool call arguments for `{}`: {e}",
                                tc.name
                            ),
                        }]
                    }
                }
            } else {
                Vec::new()
            }
        }

        AnthropicEvent::MessageDelta { delta, usage } => {
            let finish_reason = map_stop_reason(delta.stop_reason.as_deref());

            let usage = usage.map(|u| {
                state.output_tokens = u.output_tokens;
                Usage {
                    prompt_tokens: None,
                    completion_tokens: Some(u.output_tokens),
                    total_tokens: None,
                }
            });

            if let Some(u) = &usage {
                return vec![
                    RustyStreamEvent::UsageDelta { usage: u.clone() },
                    RustyStreamEvent::MessageEnd {
                        finish_reason,
                        usage: None,
                    },
                ];
            }

            vec![RustyStreamEvent::MessageEnd {
                finish_reason,
                usage: None,
            }]
        }

        AnthropicEvent::MessageStop => Vec::new(),

        AnthropicEvent::Ping => Vec::new(),

        AnthropicEvent::Error { error } => {
            vec![RustyStreamEvent::Error {
                error: format!("{}: {}", error.error_type, error.message),
            }]
        }
    }
}
