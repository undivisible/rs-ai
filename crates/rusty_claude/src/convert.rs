use rusty_ai::content::{ContentPart, ImageData};
use rusty_ai::message::{Message, Role};
use rusty_ai::model::{GenerateOptions, ThinkingConfig};
use rusty_ai::prompt::Prompt;
use rusty_ai::structured::GenerateResult;
use rusty_ai::tool::{ToolCallRequest, ToolChoice, ToolDefinition};
use rusty_ai::types::{FinishReason, ResponseMetadata};
use rusty_ai::usage::Usage;

use crate::api_types::{
    ApiContent, ApiMessage, ApiOutputConfig, ApiOutputFormat, ApiThinkingConfig, ApiTool,
    ApiToolChoice, ContentBlock, ImageSource, MessagesRequest, MessagesResponse,
};

/// Holds the separated system prompt and non-system messages.
pub(crate) struct ConvertedPrompt {
    pub system: Option<String>,
    pub messages: Vec<ApiMessage>,
}

/// Convert a `Prompt` into Anthropic-compatible parts.
///
/// Anthropic requires the system message as a separate top-level field rather
/// than as a message in the conversation array.
pub(crate) fn convert_prompt(prompt: Prompt) -> ConvertedPrompt {
    let messages = prompt.into_messages();

    let mut system_parts: Vec<String> = Vec::new();
    let mut api_messages: Vec<ApiMessage> = Vec::new();

    for msg in messages {
        match msg.role {
            Role::System => {
                for part in &msg.content {
                    if let ContentPart::Text { text } = part {
                        system_parts.push(text.clone());
                    }
                }
            }
            Role::User => {
                let content = convert_content_parts(&msg.content);
                api_messages.push(ApiMessage {
                    role: "user".to_string(),
                    content,
                });
            }
            Role::Assistant => {
                let content = convert_assistant_content(&msg);
                api_messages.push(ApiMessage {
                    role: "assistant".to_string(),
                    content,
                });
            }
            Role::Tool => {
                // Tool results in Anthropic are sent as a user message with
                // tool_result content blocks.
                let blocks = convert_tool_result_content(&msg.content);
                api_messages.push(ApiMessage {
                    role: "user".to_string(),
                    content: ApiContent::Blocks(blocks),
                });
            }
        }
    }

    let system = if system_parts.is_empty() {
        None
    } else {
        Some(system_parts.join("\n"))
    };

    ConvertedPrompt {
        system,
        messages: api_messages,
    }
}

/// Convert a list of `ContentPart` values into an `ApiContent`.
fn convert_content_parts(parts: &[ContentPart]) -> ApiContent {
    // Fast path: single text part -> use the simple string variant.
    if parts.len() == 1 {
        if let ContentPart::Text { text } = &parts[0] {
            return ApiContent::Text(text.clone());
        }
    }

    let blocks: Vec<ContentBlock> = parts
        .iter()
        .filter_map(|part| match part {
            ContentPart::Text { text } => Some(ContentBlock::Text { text: text.clone() }),
            ContentPart::Image { data } => Some(convert_image(data)),
            _ => None,
        })
        .collect();

    ApiContent::Blocks(blocks)
}

/// Convert assistant message content, which may include tool calls.
fn convert_assistant_content(msg: &Message) -> ApiContent {
    let blocks: Vec<ContentBlock> = msg
        .content
        .iter()
        .filter_map(|part| match part {
            ContentPart::Text { text } => Some(ContentBlock::Text { text: text.clone() }),
            ContentPart::ToolCall { call } => Some(ContentBlock::ToolUse {
                id: call.id.clone(),
                name: call.name.clone(),
                input: call.arguments.clone(),
            }),
            _ => None,
        })
        .collect();

    ApiContent::Blocks(blocks)
}

/// Convert tool result content parts into ContentBlock::ToolResult blocks.
fn convert_tool_result_content(parts: &[ContentPart]) -> Vec<ContentBlock> {
    parts
        .iter()
        .filter_map(|part| match part {
            ContentPart::ToolResult { result } => Some(ContentBlock::ToolResult {
                tool_use_id: result.call_id.clone(),
                content: result.content.clone(),
                is_error: if result.is_error { Some(true) } else { None },
            }),
            _ => None,
        })
        .collect()
}

fn convert_image(data: &ImageData) -> ContentBlock {
    match data {
        ImageData::Base64 { media_type, data } => ContentBlock::Image {
            source: ImageSource::Base64 {
                media_type: media_type.clone(),
                data: data.clone(),
            },
        },
        ImageData::Url { url, .. } => ContentBlock::Image {
            source: ImageSource::Url { url: url.clone() },
        },
    }
}

/// Convert `ToolDefinition` values to `ApiTool` values.
pub(crate) fn convert_tools(tools: &[ToolDefinition]) -> Vec<ApiTool> {
    tools
        .iter()
        .map(|t| ApiTool {
            name: t.name.clone(),
            description: t.description.clone(),
            input_schema: t.parameters.clone(),
        })
        .collect()
}

/// Convert `ToolChoice` to `ApiToolChoice`.
pub(crate) fn convert_tool_choice(choice: &ToolChoice) -> ApiToolChoice {
    match choice {
        ToolChoice::Auto => ApiToolChoice {
            choice_type: "auto".to_string(),
            name: None,
        },
        ToolChoice::None => ApiToolChoice {
            choice_type: "none".to_string(),
            name: None,
        },
        ToolChoice::Required => ApiToolChoice {
            choice_type: "any".to_string(),
            name: None,
        },
        ToolChoice::Specific(name) => ApiToolChoice {
            choice_type: "tool".to_string(),
            name: Some(name.clone()),
        },
    }
}

/// Build a `MessagesRequest` from the prompt and options.
pub(crate) fn build_request(
    model: &str,
    prompt: Prompt,
    options: &GenerateOptions,
    stream: bool,
) -> MessagesRequest {
    let converted = convert_prompt(prompt);
    let max_tokens = options.max_tokens.unwrap_or(4096);

    let tools = options
        .tools
        .as_ref()
        .filter(|t| !t.is_empty())
        .map(|t| convert_tools(t));

    let tool_choice = if tools.is_some() {
        options
            .tool_choice
            .as_ref()
            .map(convert_tool_choice)
            .or_else(|| {
                Some(ApiToolChoice {
                    choice_type: "auto".to_string(),
                    name: None,
                })
            })
    } else {
        None
    };

    let stop_sequences = if options.stop_sequences.is_empty() {
        None
    } else {
        Some(options.stop_sequences.clone())
    };

    let thinking = options.thinking.as_ref().map(|t| match t {
        ThinkingConfig::Adaptive => ApiThinkingConfig {
            thinking_type: "adaptive".to_string(),
        },
        ThinkingConfig::Enabled => ApiThinkingConfig {
            thinking_type: "enabled".to_string(),
        },
        ThinkingConfig::Budget { .. } => ApiThinkingConfig {
            thinking_type: "adaptive".to_string(),
        },
    });

    let output_config = options.output_schema.as_ref().map(|schema| ApiOutputConfig {
        format: ApiOutputFormat {
            format_type: "json_schema".to_string(),
            schema: schema.as_value().clone(),
        },
    });

    MessagesRequest {
        model: model.to_string(),
        max_tokens,
        messages: converted.messages,
        system: converted.system,
        temperature: options.temperature,
        top_p: options.top_p,
        top_k: options.top_k,
        stop_sequences,
        tools,
        tool_choice,
        stream,
        thinking,
        output_config,
    }
}

/// Map the Anthropic stop_reason string to `FinishReason`.
pub(crate) fn map_stop_reason(reason: Option<&str>) -> FinishReason {
    match reason {
        Some("end_turn") | Some("stop") => FinishReason::Stop,
        Some("max_tokens") => FinishReason::Length,
        Some("tool_use") => FinishReason::ToolCall,
        Some("content_filter") => FinishReason::ContentFilter,
        _ => FinishReason::Unknown,
    }
}

/// Convert a `MessagesResponse` (non-streaming) into a `GenerateResult`.
pub(crate) fn convert_response(response: MessagesResponse) -> GenerateResult {
    let mut text_parts: Vec<String> = Vec::new();
    let mut tool_calls: Vec<ToolCallRequest> = Vec::new();

    for block in &response.content {
        match block {
            ContentBlock::Text { text } => {
                text_parts.push(text.clone());
            }
            ContentBlock::ToolUse { id, name, input } => {
                tool_calls.push(ToolCallRequest {
                    id: id.clone(),
                    name: name.clone(),
                    arguments: input.clone(),
                });
            }
            _ => {}
        }
    }

    let text = if text_parts.is_empty() {
        None
    } else {
        Some(text_parts.join(""))
    };

    let finish_reason = map_stop_reason(response.stop_reason.as_deref());

    let usage = Usage {
        prompt_tokens: Some(response.usage.input_tokens),
        completion_tokens: Some(response.usage.output_tokens),
        total_tokens: Some(response.usage.input_tokens + response.usage.output_tokens),
    };

    GenerateResult {
        text,
        tool_calls,
        finish_reason,
        usage,
        metadata: ResponseMetadata {
            provider: "anthropic".to_string(),
            model: response.model,
            ..ResponseMetadata::default()
        },
    }
}
