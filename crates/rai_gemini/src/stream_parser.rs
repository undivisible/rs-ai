use crate::api_types::*;
use crate::convert::{map_finish_reason, map_usage};
use futures::stream::{self, StreamExt};
use reqwest::Response;
use rai_ai::error::AiError;
use rai_ai::stream::{AiStream, StreamEvent};

/// Parse Gemini's Server-Sent Events streaming format into an `AiStream`.
///
/// The streaming endpoint (with `alt=sse`) returns standard SSE where each
/// `data:` line contains a JSON `GenerateContentResponse` object.
pub(crate) fn parse_stream(response: Response) -> AiStream {
    let byte_stream = response.bytes_stream();

    // Phase 1: Parse raw bytes into SSE data payloads (JSON strings).
    let json_stream = futures::stream::unfold(
        (byte_stream, String::new()),
        |(mut byte_stream, mut buffer)| async move {
            loop {
                // Try to extract a complete SSE data line from the buffer.
                if let Some(json_str) = extract_next_data_line(&mut buffer) {
                    return Some((Ok(json_str), (byte_stream, buffer)));
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
                            Err(AiError::Transport {
                                message: msg,
                                source: Some(Box::new(e)),
                            }),
                            (byte_stream, buffer),
                        ));
                    }
                    None => {
                        // End of stream. Try to parse any remaining data lines.
                        if let Some(json_str) = extract_next_data_line(&mut buffer) {
                            return Some((Ok(json_str), (byte_stream, buffer)));
                        }
                        return None;
                    }
                }
            }
        },
    );

    // Phase 2: Parse JSON strings into GenerateContentResponse and then into
    // StreamEvents, emitting MessageStart at the beginning.
    let event_stream = futures::stream::unfold(
        (Box::pin(json_stream), false),
        |(mut json_stream, mut sent_start)| async move {
            loop {
                // Emit MessageStart before the first real event.
                if !sent_start {
                    sent_start = true;
                    let start_event: Vec<Result<StreamEvent, AiError>> =
                        vec![Ok(StreamEvent::MessageStart {
                            message_id: uuid::Uuid::new_v4().to_string(),
                        })];
                    return Some((stream::iter(start_event), (json_stream, sent_start)));
                }

                match json_stream.next().await {
                    Some(Ok(json_str)) => {
                        match serde_json::from_str::<GenerateContentResponse>(&json_str) {
                            Ok(response) => {
                                let events = response_to_stream_events(response);
                                if !events.is_empty() {
                                    let items: Vec<Result<StreamEvent, AiError>> =
                                        events.into_iter().map(Ok).collect();
                                    return Some((stream::iter(items), (json_stream, sent_start)));
                                }
                                continue;
                            }
                            Err(e) => {
                                tracing::error!(
                                    data = %json_str,
                                    error = %e,
                                    "Failed to parse Gemini SSE event; terminating stream"
                                );
                                return Some((
                                    stream::iter(vec![Err(AiError::StreamError {
                                        message: format!("Unparseable SSE event from Gemini: {e}"),
                                    })]),
                                    (json_stream, sent_start),
                                ));
                            }
                        }
                    }
                    Some(Err(e)) => {
                        return Some((stream::iter(vec![Err(e)]), (json_stream, sent_start)));
                    }
                    None => return None,
                }
            }
        },
    )
    .flat_map(|items| items);

    Box::pin(event_stream)
}

/// Extract the next complete `data: ...` line from the buffer.
/// Removes the consumed portion from the buffer.
fn extract_next_data_line(buffer: &mut String) -> Option<String> {
    loop {
        let newline_pos = buffer.find('\n')?;
        let line = buffer[..newline_pos].to_string();
        *buffer = buffer[newline_pos + 1..].to_string();

        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(':') {
            continue;
        }

        if let Some(data) = trimmed.strip_prefix("data:") {
            let data = data.trim();
            if !data.is_empty() {
                return Some(data.to_string());
            }
        }
        // Skip non-data SSE fields (event:, id:, retry:).
    }
}

/// Convert a single Gemini response chunk into stream events.
fn response_to_stream_events(response: GenerateContentResponse) -> Vec<StreamEvent> {
    let mut events = Vec::new();

    if let Some(candidates) = &response.candidates {
        if let Some(candidate) = candidates.first() {
            if let Some(ref content) = candidate.content {
                for part in &content.parts {
                    match part {
                        GeminiPart::Text { text } => {
                            events.push(StreamEvent::TextDelta {
                                delta: text.clone(),
                            });
                        }
                        GeminiPart::Thought { text, .. } => {
                            events.push(StreamEvent::ThinkingDelta {
                                delta: text.clone(),
                            });
                        }
                        GeminiPart::FunctionCall { function_call } => {
                            let call_id = uuid::Uuid::new_v4().to_string();
                            events.push(StreamEvent::ToolCallStart {
                                call_id: call_id.clone(),
                                tool_name: function_call.name.clone(),
                            });
                            events.push(StreamEvent::ToolCallEnd {
                                call_id,
                                arguments: function_call.args.clone(),
                            });
                        }
                        _ => {}
                    }
                }
            }

            if let Some(ref reason) = candidate.finish_reason {
                let finish = map_finish_reason(reason);
                let usage = map_usage(response.usage_metadata.as_ref());
                events.push(StreamEvent::MessageEnd {
                    finish_reason: finish,
                    usage: Some(usage),
                });
            }
        }
    }

    // Emit usage if present and no MessageEnd was emitted yet.
    if let Some(ref meta) = response.usage_metadata {
        let already_has_end = events
            .iter()
            .any(|e| matches!(e, StreamEvent::MessageEnd { .. }));
        if !already_has_end {
            events.push(StreamEvent::UsageDelta {
                usage: map_usage(Some(meta)),
            });
        }
    }

    events
}
