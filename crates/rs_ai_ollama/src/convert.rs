use rs_ai_traits::{
    ContentPart, GenerateOptions, ImageData, Message, Role, ToolCallRequest, ToolDefinition,
};

use crate::api_types::{
    OllamaFunction, OllamaFunctionCall, OllamaMessage, OllamaOptions, OllamaTool, OllamaToolCall,
};

/// Convert a slice of `rs_ai_traits::Message` into Ollama messages.
pub(crate) fn convert_messages(messages: &[Message]) -> Vec<OllamaMessage> {
    messages.iter().map(convert_message).collect()
}

fn convert_message(msg: &Message) -> OllamaMessage {
    let role = match msg.role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    };

    let mut text_parts: Vec<String> = Vec::new();
    let mut images: Vec<String> = Vec::new();
    let mut tool_calls: Vec<OllamaToolCall> = Vec::new();

    for part in &msg.content {
        match part {
            ContentPart::Text { text } => {
                text_parts.push(text.clone());
            }
            ContentPart::Image { data } => {
                match data {
                    ImageData::Base64 { data, .. } => {
                        images.push(data.clone());
                    }
                    ImageData::Url { url, .. } => {
                        // Ollama only supports base64 images natively. Pass the URL as text
                        // as a fallback; real usage should provide base64-encoded images.
                        text_parts.push(format!("[image: {}]", url));
                    }
                }
            }
            ContentPart::ToolCall { call } => {
                tool_calls.push(OllamaToolCall {
                    function: OllamaFunctionCall {
                        name: call.name.clone(),
                        arguments: call.arguments.clone(),
                    },
                });
            }
            ContentPart::ToolResult { result } => {
                text_parts.push(result.content.clone());
            }
            ContentPart::File { .. } => {
                // Ollama does not support file parts; skip.
            }
        }
    }

    OllamaMessage {
        role: role.to_string(),
        content: text_parts.join("\n"),
        images: if images.is_empty() {
            None
        } else {
            Some(images)
        },
        tool_calls: if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        },
        thinking: None,
    }
}

/// Convert `GenerateOptions` into `OllamaOptions` (returns `None` if nothing is set).
pub(crate) fn convert_options(opts: &GenerateOptions) -> Option<OllamaOptions> {
    let o = OllamaOptions {
        temperature: opts.temperature,
        num_predict: opts.max_tokens,
        top_p: opts.top_p,
        top_k: opts.top_k,
        stop: if opts.stop_sequences.is_empty() {
            None
        } else {
            Some(opts.stop_sequences.clone())
        },
        seed: opts.seed,
        frequency_penalty: opts.frequency_penalty,
        presence_penalty: opts.presence_penalty,
    };

    // Check if everything is None / empty.
    if o.temperature.is_none()
        && o.num_predict.is_none()
        && o.top_p.is_none()
        && o.top_k.is_none()
        && o.stop.is_none()
        && o.seed.is_none()
        && o.frequency_penalty.is_none()
        && o.presence_penalty.is_none()
    {
        return None;
    }
    Some(o)
}

/// Convert `rs_ai_traits::ToolDefinition` to Ollama tools.
pub(crate) fn convert_tools(tools: Option<&[ToolDefinition]>) -> Option<Vec<OllamaTool>> {
    let tools = tools?;
    if tools.is_empty() {
        return None;
    }
    Some(
        tools
            .iter()
            .map(|t| OllamaTool {
                tool_type: "function".to_string(),
                function: OllamaFunction {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    parameters: t.parameters.clone(),
                },
            })
            .collect(),
    )
}

/// Convert Ollama tool calls to `rs_ai_traits::ToolCallRequest`.
pub(crate) fn convert_tool_calls(calls: &[OllamaToolCall]) -> Vec<ToolCallRequest> {
    calls
        .iter()
        .enumerate()
        .map(|(i, tc)| ToolCallRequest {
            id: format!("call_{}", i),
            name: tc.function.name.clone(),
            arguments: tc.function.arguments.clone(),
        })
        .collect()
}
