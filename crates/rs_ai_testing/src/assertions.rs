//! Test helper functions for asserting on AI SDK types.

use futures::StreamExt;

use rs_ai_core::error::AiResult;
use rs_ai_core::stream::{AiStream, StreamEvent};
use rs_ai_core::structured::GenerateResult;

/// Collect all text deltas from a stream into a single string.
pub async fn collect_text(mut stream: AiStream) -> AiResult<String> {
    let mut text = String::new();
    while let Some(event) = stream.next().await {
        let event = event?;
        if let StreamEvent::TextDelta { delta } = event {
            text.push_str(&delta);
        }
    }
    Ok(text)
}

/// Collect all events from a stream into a vector.
pub async fn collect_events(mut stream: AiStream) -> AiResult<Vec<StreamEvent>> {
    let mut events = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event?);
    }
    Ok(events)
}

/// Assert that a stream produces the expected text (all text deltas concatenated).
///
/// # Panics
///
/// Panics if the stream produces an error or the collected text does not match.
pub async fn assert_stream_text(stream: AiStream, expected: &str) {
    let text = collect_text(stream)
        .await
        .expect("stream should not produce an error");
    assert_eq!(
        text, expected,
        "stream text mismatch: expected {:?}, got {:?}",
        expected, text
    );
}

/// Assert that a `GenerateResult` contains text matching the expected value.
///
/// # Panics
///
/// Panics if the result has no text or the text does not match.
pub fn assert_result_text(result: &GenerateResult, expected: &str) {
    let text = result
        .text
        .as_deref()
        .expect("GenerateResult should contain text");
    assert_eq!(
        text, expected,
        "result text mismatch: expected {:?}, got {:?}",
        expected, text
    );
}

/// Assert that a `GenerateResult` contains tool calls with the given names.
///
/// The order of `tool_names` does not matter; all listed names must be present.
///
/// # Panics
///
/// Panics if any of the expected tool names are missing.
pub fn assert_has_tool_calls(result: &GenerateResult, tool_names: &[&str]) {
    let actual_names: Vec<&str> = result.tool_calls.iter().map(|c| c.name.as_str()).collect();
    for expected in tool_names {
        assert!(
            actual_names.contains(expected),
            "expected tool call {:?} not found in {:?}",
            expected,
            actual_names,
        );
    }
}
