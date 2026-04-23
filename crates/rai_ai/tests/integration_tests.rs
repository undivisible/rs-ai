//! Integration tests for rai_ai core functionality

#[cfg(test)]
mod tests {
    use rai_ai::*;

    #[test]
    fn test_finish_reason_variants() {
        // Test that finish reason enum is properly exported
        let _reason = FinishReason::Stop;
        let _reason = FinishReason::Length;
        let _reason = FinishReason::ToolUse;
        assert!(true);
    }

    #[test]
    fn test_role_variants() {
        // Test message role enum
        let _role = Role::User;
        let _role = Role::Assistant;
        let _role = Role::System;
        assert!(true);
    }

    #[test]
    fn test_content_part_variants() {
        use rai_ai::ContentPart;

        let _text = ContentPart::Text("Hello".to_string());
        let _image = ContentPart::ImageUrl {
            url: "https://example.com/image.jpg".to_string(),
            detail: ImageDetail::Low,
        };
        assert!(true);
    }

    #[test]
    fn test_tool_definition_creation() {
        use rai_ai::ToolDefinition;
        use schemars::schema_for;
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, schemars::JsonSchema)]
        struct SimpleInput {
            text: String,
        }

        let tool = ToolDefinition {
            name: "echo".to_string(),
            description: "Echo the input".to_string(),
            input_schema: schema_for!(SimpleInput),
        };

        assert_eq!(tool.name, "echo");
        assert_eq!(tool.description, "Echo the input");
    }

    #[test]
    fn test_message_creation() {
        use rai_ai::{ContentPart, Message, Role};

        let message = Message {
            role: Role::User,
            content: vec![ContentPart::Text("Hello, world!".to_string())],
        };

        assert_eq!(message.role, Role::User);
        assert_eq!(message.content.len(), 1);
    }

    #[test]
    fn test_capability_set() {
        use rai_ai::{Capability, CapabilitySet};

        let mut capabilities = CapabilitySet::new();
        capabilities.add(Capability::TextGeneration);
        capabilities.add(Capability::ToolUse);

        assert!(capabilities.supports(&Capability::TextGeneration));
        assert!(capabilities.supports(&Capability::ToolUse));
        assert!(!capabilities.supports(&Capability::Vision));
    }

    #[test]
    fn test_usage_calculation() {
        use rai_ai::Usage;

        let usage = Usage {
            input_tokens: 10,
            output_tokens: 20,
        };

        assert_eq!(usage.total_tokens(), 30);
    }

    #[test]
    fn test_image_detail_variants() {
        use rai_ai::ImageDetail;

        let _low = ImageDetail::Low;
        let _high = ImageDetail::High;
        let _auto = ImageDetail::Auto;
        assert!(true);
    }

    #[test]
    fn test_output_schema_creation() {
        use rai_ai::OutputSchema;
        use schemars::schema_for;
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, schemars::JsonSchema)]
        struct BookReview {
            title: String,
            rating: u8,
        }

        let schema = OutputSchema {
            json_schema: Some(schema_for!(BookReview)),
        };

        assert!(schema.json_schema.is_some());
    }

    #[test]
    fn test_tool_choice_variants() {
        use rai_ai::ToolChoice;

        let _auto = ToolChoice::Auto;
        let _required = ToolChoice::Required;
        let _disabled = ToolChoice::Disabled;
        assert!(true);
    }

    #[test]
    fn test_error_types() {
        use rai_ai::{AiError, AiResult};

        // Test provider error
        let err: AiResult<String> = Err(AiError::ProviderError {
            provider: "test".to_string(),
            status: Some(500),
            message: "Server error".to_string(),
        });
        assert!(err.is_err());

        // Test auth error
        let err: AiResult<String> = Err(AiError::AuthError {
            message: "Invalid API key".to_string(),
        });
        assert!(err.is_err());

        // Test timeout
        let err: AiResult<String> = Err(AiError::Timeout);
        assert!(err.is_err());

        // Test unsupported capability
        let err: AiResult<String> = Err(AiError::UnsupportedCapability {
            capability: "vision".to_string(),
            provider: "test".to_string(),
        });
        assert!(err.is_err());
    }

    #[test]
    fn test_streaming_event_types() {
        use rai_ai::{StreamEvent, Usage};

        let text_delta = StreamEvent::TextDelta {
            delta: "Hello".to_string(),
        };
        assert!(matches!(text_delta, StreamEvent::TextDelta { .. }));

        let message_end = StreamEvent::MessageEnd {
            finish_reason: FinishReason::Stop,
            usage: Some(Usage {
                input_tokens: 5,
                output_tokens: 10,
            }),
        };
        assert!(matches!(message_end, StreamEvent::MessageEnd { .. }));
    }

    #[test]
    fn test_model_info() {
        use rai_ai::ModelInfo;

        let model = ModelInfo {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            description: Some("Advanced model".to_string()),
            context_window: Some(8192),
        };

        assert_eq!(model.id, "gpt-4");
        assert_eq!(model.name, "GPT-4");
    }

    #[test]
    fn test_prompt_conversion() {
        use rai_ai::Prompt;

        let _text_prompt = Prompt::Text("Hello".to_string());
        let _message_prompt = Prompt::Messages(vec![]);
        assert!(true);
    }

    #[test]
    fn test_generate_options_defaults() {
        use rai_ai::GenerateOptions;

        let opts = GenerateOptions::default();
        assert_eq!(opts.max_tokens, None);
        assert_eq!(opts.temperature, None);
    }

    #[test]
    fn test_generate_options_builder() {
        use rai_ai::GenerateOptions;

        let opts = GenerateOptions::default()
            .with_temperature(0.7)
            .with_max_tokens(100);

        assert_eq!(opts.temperature, Some(0.7));
        assert_eq!(opts.max_tokens, Some(100));
    }

    #[test]
    fn test_synthetic_streamer() {
        use rai_ai::{StreamEvent, SyntheticStreamer};

        let events = vec![
            StreamEvent::TextDelta {
                delta: "Hello".to_string(),
            },
            StreamEvent::TextDelta {
                delta: " world".to_string(),
            },
            StreamEvent::MessageEnd {
                finish_reason: FinishReason::Stop,
                usage: None,
            },
        ];

        let _streamer = SyntheticStreamer::new(events);
        assert!(true);
    }
}
