use crate::StreamEvent;
use serde::{Deserialize, Serialize};

/// The protocol version for the UI stream format.
pub const PROTOCOL_VERSION: &str = "1.0";

/// A UI-facing stream event. This is a versioned, frontend-friendly
/// representation of the core [`StreamEvent`] type.
///
/// Events are tagged with `"type"` for easy dispatch on the client side.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UiStreamEvent {
    /// Signals that generation has started.
    #[serde(rename = "start")]
    Start {
        /// Unique message identifier.
        message_id: String,
        /// Model identifier.
        model: String,
        /// Protocol version.
        version: String,
    },

    /// A chunk of generated text.
    #[serde(rename = "text")]
    Text {
        /// Text delta.
        delta: String,
    },

    /// A tool call has begun.
    #[serde(rename = "tool_call_start")]
    ToolCallStart {
        /// Call identifier.
        call_id: String,
        /// Tool name.
        tool_name: String,
    },

    /// A chunk of tool call arguments (partial JSON).
    #[serde(rename = "tool_call_args")]
    ToolCallArgs {
        /// Call identifier.
        call_id: String,
        /// Argument JSON delta.
        delta: String,
    },

    /// A tool call has finished accumulating arguments.
    #[serde(rename = "tool_call_end")]
    ToolCallEnd {
        /// Call identifier.
        call_id: String,
    },

    /// Result from executing a tool.
    #[serde(rename = "tool_result")]
    ToolResult {
        /// Call identifier.
        call_id: String,
        /// Tool result content.
        content: String,
        /// Whether the result is an error.
        is_error: bool,
    },

    /// A partial structured-output object.
    #[serde(rename = "object")]
    Object {
        /// Object delta.
        delta: serde_json::Value,
    },

    /// Token usage information.
    #[serde(rename = "usage")]
    Usage {
        /// Prompt token count.
        prompt_tokens: Option<u64>,
        /// Completion token count.
        completion_tokens: Option<u64>,
    },

    /// An error occurred during generation.
    #[serde(rename = "error")]
    Error {
        /// Error code.
        code: String,
        /// Error message.
        message: String,
    },

    /// Intermediate thinking / reasoning tokens from a reasoning model.
    #[serde(rename = "thinking")]
    Thinking {
        /// Thinking delta.
        delta: String,
    },

    /// Generation is complete.
    #[serde(rename = "done")]
    Done {
        /// Finish reason.
        finish_reason: String,
    },
}

impl From<StreamEvent> for UiStreamEvent {
    fn from(event: StreamEvent) -> Self {
        match event {
            StreamEvent::MessageStart { message_id } => UiStreamEvent::Start {
                message_id,
                model: String::new(),
                version: PROTOCOL_VERSION.to_string(),
            },
            StreamEvent::TextDelta { delta } => UiStreamEvent::Text { delta },
            StreamEvent::ToolCallStart { call_id, tool_name } => {
                UiStreamEvent::ToolCallStart { call_id, tool_name }
            }
            StreamEvent::ToolCallDelta { call_id, delta } => {
                UiStreamEvent::ToolCallArgs { call_id, delta }
            }
            StreamEvent::ToolCallEnd { call_id, .. } => UiStreamEvent::ToolCallEnd { call_id },
            StreamEvent::ToolResult {
                call_id,
                content,
                is_error,
            } => UiStreamEvent::ToolResult {
                call_id,
                content,
                is_error,
            },
            StreamEvent::ObjectDelta { delta } => UiStreamEvent::Object { delta },
            StreamEvent::UsageDelta { usage } => UiStreamEvent::Usage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
            },
            StreamEvent::Warning { message } => UiStreamEvent::Error {
                code: "warning".to_string(),
                message,
            },
            StreamEvent::MessageEnd {
                finish_reason,
                usage: _,
            } => {
                let reason = format!("{:?}", finish_reason).to_lowercase();
                UiStreamEvent::Done {
                    finish_reason: reason,
                }
            }
            StreamEvent::Error { error } => UiStreamEvent::Error {
                code: "stream_error".to_string(),
                message: error,
            },
            StreamEvent::ThinkingDelta { delta } => UiStreamEvent::Thinking { delta },
            StreamEvent::SyntheticStreamingNotice => UiStreamEvent::Error {
                code: "notice".to_string(),
                message: "synthetic streaming".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FinishReason;

    #[test]
    fn test_start_conversion() {
        let stream_event = StreamEvent::MessageStart {
            message_id: "msg-123".into(),
        };
        let ui_event: UiStreamEvent = stream_event.into();
        match ui_event {
            UiStreamEvent::Start {
                message_id,
                version,
                ..
            } => {
                assert_eq!(message_id, "msg-123");
                assert_eq!(version, PROTOCOL_VERSION);
            }
            _ => panic!("expected Start event"),
        }
    }

    #[test]
    fn test_text_delta_conversion() {
        let stream_event = StreamEvent::TextDelta {
            delta: "Hello".into(),
        };
        let ui_event: UiStreamEvent = stream_event.into();
        match ui_event {
            UiStreamEvent::Text { delta } => assert_eq!(delta, "Hello"),
            _ => panic!("expected Text event"),
        }
    }

    #[test]
    fn test_serde_roundtrip() {
        let event = UiStreamEvent::Error {
            code: "rate_limit".into(),
            message: "Too many requests".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: UiStreamEvent = serde_json::from_str(&json).unwrap();
        match deserialized {
            UiStreamEvent::Error { code, message } => {
                assert_eq!(code, "rate_limit");
                assert_eq!(message, "Too many requests");
            }
            _ => panic!("expected Error event"),
        }
    }

    #[test]
    fn test_done_conversion() {
        let stream_event = StreamEvent::MessageEnd {
            finish_reason: FinishReason::Stop,
            usage: None,
        };
        let ui_event: UiStreamEvent = stream_event.into();
        match ui_event {
            UiStreamEvent::Done { finish_reason } => assert_eq!(finish_reason, "stop"),
            _ => panic!("expected Done event"),
        }
    }
}
