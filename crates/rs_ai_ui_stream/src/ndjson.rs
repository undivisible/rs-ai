use bytes::Bytes;
use futures::Stream;
use rs_ai_traits::{AiError, AiStream};
use tokio_stream::StreamExt;

use crate::UiStreamEvent;

/// Encoder for the [NDJSON](http://ndjson.org/) (Newline-Delimited JSON)
/// wire format.
///
/// Each event is serialized as a single line of JSON followed by a newline:
/// ```text
/// {"type":"text","delta":"Hello"}\n
/// ```
pub struct NdjsonEncoder;

impl NdjsonEncoder {
    /// Encode a single [`UiStreamEvent`] as NDJSON (one JSON object followed
    /// by `\n`).
    pub fn encode(event: &UiStreamEvent) -> String {
        let json = serde_json::to_string(event).unwrap_or_else(|e| {
            format!(
                r#"{{"type":"error","code":"serialization_error","message":"{}"}}"#,
                e.to_string().replace('"', "\\\"")
            )
        });
        format!("{json}\n")
    }

    /// Transform an [`AiStream`] into a stream of NDJSON-encoded [`Bytes`]
    /// chunks.
    ///
    /// Each `Ok` item from the source stream is converted to a
    /// [`UiStreamEvent`] and then NDJSON-encoded. Errors from the source
    /// stream are converted into [`UiStreamEvent::Error`] events so the
    /// client always receives well-formed NDJSON.
    pub fn encode_stream(stream: AiStream) -> impl Stream<Item = Result<Bytes, AiError>> {
        stream.map(|result| match result {
            Ok(event) => {
                let ui_event: UiStreamEvent = event.into();
                let encoded = Self::encode(&ui_event);
                Ok(Bytes::from(encoded))
            }
            Err(e) => {
                let error_event = UiStreamEvent::Error {
                    code: "stream_error".to_string(),
                    message: e.to_string(),
                };
                let encoded = Self::encode(&error_event);
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
            delta: "hello".into(),
        };
        let ndjson = NdjsonEncoder::encode(&event);
        assert!(ndjson.ends_with('\n'));
        // Should be exactly one line.
        assert_eq!(ndjson.matches('\n').count(), 1);
        let parsed: UiStreamEvent = serde_json::from_str(ndjson.trim()).unwrap();
        match parsed {
            UiStreamEvent::Text { delta } => assert_eq!(delta, "hello"),
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
        let ndjson = NdjsonEncoder::encode(&event);
        assert!(ndjson.contains(r#""type":"start""#));
        assert!(ndjson.ends_with('\n'));
    }

    #[tokio::test]
    async fn test_encode_stream() {
        use futures::stream;
        use rs_ai_traits::StreamEvent;

        use tokio_stream::StreamExt;

        let events: Vec<Result<StreamEvent, AiError>> = vec![
            Ok(StreamEvent::MessageStart {
                message_id: "m1".into(),
            }),
            Ok(StreamEvent::TextDelta { delta: "Hi".into() }),
            Ok(StreamEvent::MessageEnd {
                finish_reason: rs_ai_traits::FinishReason::Stop,
                usage: None,
            }),
        ];

        let ai_stream: AiStream = Box::pin(stream::iter(events));

        let encoded: Vec<_> = NdjsonEncoder::encode_stream(ai_stream).collect().await;

        assert_eq!(encoded.len(), 3);
        for item in &encoded {
            assert!(item.is_ok());
            let bytes = item.as_ref().unwrap();
            let s = std::str::from_utf8(bytes).unwrap();
            assert!(s.ends_with('\n'));
            // Each line should be valid JSON.
            let _: serde_json::Value = serde_json::from_str(s.trim()).unwrap();
        }
    }

    #[tokio::test]
    async fn test_encode_stream_with_error() {
        use futures::stream;
        use rs_ai_traits::StreamEvent;

        use tokio_stream::StreamExt;

        let events: Vec<Result<StreamEvent, AiError>> = vec![
            Ok(StreamEvent::TextDelta { delta: "Hi".into() }),
            Err(AiError::StreamError {
                message: "timeout".into(),
            }),
        ];

        let ai_stream: AiStream = Box::pin(stream::iter(events));

        let encoded: Vec<_> = NdjsonEncoder::encode_stream(ai_stream).collect().await;

        assert_eq!(encoded.len(), 2);
        assert!(encoded[1].is_ok());
        let bytes = encoded[1].as_ref().unwrap();
        let s = std::str::from_utf8(bytes).unwrap();
        assert!(s.contains("stream_error"));
        assert!(s.contains("timeout"));
    }
}
