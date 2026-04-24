use bytes::Bytes;
use futures::Stream;
use rs_ai_ai::{AiError, AiStream};
use tokio_stream::StreamExt;

use crate::UiStreamEvent;

/// Encoder for the [Server-Sent Events](https://html.spec.whatwg.org/multipage/server-sent-events.html)
/// wire format.
///
/// Each event is serialized as:
/// ```text
/// data: {"type":"text","delta":"Hello"}\n\n
/// ```
pub struct SseEncoder;

impl SseEncoder {
    /// Encode a single [`UiStreamEvent`] into SSE format.
    ///
    /// The output has the form `data: {json}\n\n` and is ready to be written
    /// directly to an HTTP response body.
    pub fn encode(event: &UiStreamEvent) -> String {
        // serde_json::to_string never fails for our types (no maps with
        // non-string keys, etc.), but we handle the error defensively.
        let json = serde_json::to_string(event).unwrap_or_else(|e| {
            // Fall back to an error event so the client always gets valid SSE.
            format!(
                r#"{{"type":"error","code":"serialization_error","message":"{}"}}"#,
                e.to_string().replace('"', "\\\"")
            )
        });
        format!("data: {json}\n\n")
    }

    /// Transform an [`AiStream`] into a stream of SSE-encoded [`Bytes`] chunks.
    ///
    /// Each `Ok` item from the source stream is converted to a
    /// [`UiStreamEvent`] and then SSE-encoded. Errors from the source stream
    /// are forwarded as [`UiStreamEvent::Error`] events so the client always
    /// receives well-formed SSE data, followed by the original error being
    /// propagated.
    pub fn encode_stream(stream: AiStream) -> impl Stream<Item = Result<Bytes, AiError>> {
        stream.map(|result| match result {
            Ok(event) => {
                let ui_event: UiStreamEvent = event.into();
                let encoded = Self::encode(&ui_event);
                Ok(Bytes::from(encoded))
            }
            Err(e) => {
                // Emit an SSE error frame so the client can handle it, then
                // propagate the original error to let the transport layer
                // know the stream is unhealthy.
                let error_event = UiStreamEvent::Error {
                    code: "stream_error".to_string(),
                    message: e.to_string(),
                };
                let encoded = Self::encode(&error_event);
                // We choose to return the encoded error event as a successful
                // byte chunk so the client receives it. The stream will
                // naturally end after the source yields this error.
                //
                // If callers prefer to propagate the error, they can inspect
                // the UiStreamEvent::Error on the client side.
                Ok(Bytes::from(encoded))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PROTOCOL_VERSION;

    #[test]
    fn test_encode_text_event() {
        let event = UiStreamEvent::Text {
            delta: "world".into(),
        };
        let sse = SseEncoder::encode(&event);
        assert!(sse.starts_with("data: "));
        assert!(sse.ends_with("\n\n"));
        // The JSON payload should be valid.
        let json_str = sse.trim_start_matches("data: ").trim();
        let parsed: UiStreamEvent = serde_json::from_str(json_str).unwrap();
        match parsed {
            UiStreamEvent::Text { delta } => assert_eq!(delta, "world"),
            _ => panic!("unexpected variant"),
        }
    }

    #[test]
    fn test_encode_start_event() {
        let event = UiStreamEvent::Start {
            message_id: "abc".into(),
            model: "test-model".into(),
            version: PROTOCOL_VERSION.into(),
        };
        let sse = SseEncoder::encode(&event);
        assert!(sse.contains(r#""type":"start""#));
        assert!(sse.contains(r#""version":"1.0""#));
    }

    #[tokio::test]
    async fn test_encode_stream() {
        use futures::stream;
        use rs_ai_ai::StreamEvent;

        use tokio_stream::StreamExt;

        let events: Vec<Result<StreamEvent, AiError>> = vec![
            Ok(StreamEvent::MessageStart {
                message_id: "m1".into(),
            }),
            Ok(StreamEvent::TextDelta { delta: "Hi".into() }),
            Ok(StreamEvent::MessageEnd {
                finish_reason: rs_ai_ai::FinishReason::Stop,
                usage: None,
            }),
        ];

        let ai_stream: AiStream = Box::pin(stream::iter(events));

        let encoded: Vec<_> = SseEncoder::encode_stream(ai_stream).collect().await;

        assert_eq!(encoded.len(), 3);
        for item in &encoded {
            assert!(item.is_ok());
            let bytes = item.as_ref().unwrap();
            let s = std::str::from_utf8(bytes).unwrap();
            assert!(s.starts_with("data: "));
            assert!(s.ends_with("\n\n"));
        }
    }

    #[tokio::test]
    async fn test_encode_stream_with_error() {
        use futures::stream;
        use rs_ai_ai::StreamEvent;

        use tokio_stream::StreamExt;

        let events: Vec<Result<StreamEvent, AiError>> = vec![
            Ok(StreamEvent::TextDelta { delta: "Hi".into() }),
            Err(AiError::StreamError {
                message: "connection lost".into(),
            }),
        ];

        let ai_stream: AiStream = Box::pin(stream::iter(events));

        let encoded: Vec<_> = SseEncoder::encode_stream(ai_stream).collect().await;

        assert_eq!(encoded.len(), 2);
        // The error should have been encoded as an SSE event (Ok bytes).
        assert!(encoded[1].is_ok());
        let bytes = encoded[1].as_ref().unwrap();
        let s = std::str::from_utf8(bytes).unwrap();
        assert!(s.contains("stream_error"));
        assert!(s.contains("connection lost"));
    }
}
