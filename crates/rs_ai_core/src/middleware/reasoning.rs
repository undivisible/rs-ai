#![allow(clippy::if_same_then_else)]
//! Middleware that extracts reasoning (thinking) tokens from model responses.
//! Equivalent to Vercel AI SDK's `extractReasoningMiddleware()`.

use async_trait::async_trait;

use crate::{AiResult, GenerateOptions, GenerateResult, Middleware, MiddlewareNext, Prompt};

/// Middleware that extracts reasoning content from model output.
///
/// Reasoning content is typically enclosed in `thinking` XML tags
/// (Anthropic), `reasoning_content` fields (OpenAI), or markdown
/// code fences with `reasoning` designation.
///
/// After extraction, the reasoning text is stored in
/// `GenerateResult.reasoning` and stripped from the visible `text` field.
pub struct ExtractReasoningMiddleware;

#[async_trait]
impl Middleware for ExtractReasoningMiddleware {
    async fn process(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
        next: MiddlewareNext<'_>,
    ) -> AiResult<GenerateResult> {
        let mut result = next.run(prompt, options).await?;

        let mut reasoning_parts: Vec<String> = Vec::new();

        // Extract reasoning from <thinking> XML tags (Anthropic-style)
        if let Some(ref text) = result.text {
            if let Some(extracted) = extract_thinking_tags(text) {
                reasoning_parts.push(extracted.reasoning);
                result.text = Some(extracted.clean_text);
            }
        }

        // Also check for ```reasoning ... ``` code blocks
        if let Some(ref text) = result.text {
            if let Some(extracted) = extract_reasoning_fences(text) {
                if !reasoning_parts.is_empty() {
                    reasoning_parts.push(extracted.reasoning);
                } else {
                    reasoning_parts.push(extracted.reasoning);
                }
                result.text = Some(extracted.clean_text);
            }
        }

        if !reasoning_parts.is_empty() {
            result.reasoning = Some(reasoning_parts.join("\n---\n"));
        }

        Ok(result)
    }
}

/// Create a new [`ExtractReasoningMiddleware`].
pub fn extract_reasoning_middleware() -> ExtractReasoningMiddleware {
    ExtractReasoningMiddleware
}

struct Extracted {
    reasoning: String,
    clean_text: String,
}

/// Extract content from `<thinking>...</thinking>` tags.
fn extract_thinking_tags(text: &str) -> Option<Extracted> {
    let mut reasoning_parts = Vec::new();
    let mut clean = String::new();
    let mut remaining = text;

    while let Some(start) = remaining.find("<thinking>") {
        clean.push_str(&remaining[..start]);
        remaining = &remaining[start + "<thinking>".len()..];

        if let Some(end) = remaining.find("</thinking>") {
            reasoning_parts.push(remaining[..end].to_string());
            remaining = &remaining[end + "</thinking>".len()..];
        } else {
            // unclosed tag — treat rest as reasoning
            reasoning_parts.push(remaining.to_string());
            remaining = "";
            break;
        }
    }

    if reasoning_parts.is_empty() {
        return None;
    }

    clean.push_str(remaining);
    Some(Extracted {
        reasoning: reasoning_parts.join("\n"),
        clean_text: clean,
    })
}

/// Extract content from ```reasoning ... ``` code blocks.
fn extract_reasoning_fences(text: &str) -> Option<Extracted> {
    let mut reasoning_parts = Vec::new();
    let mut clean = String::new();
    let mut remaining = text;
    let fence_marker = "```reasoning";

    while let Some(start) = remaining.find(fence_marker) {
        clean.push_str(&remaining[..start]);
        remaining = &remaining[start + fence_marker.len()..];

        if let Some(end) = remaining.find("```") {
            reasoning_parts.push(remaining[..end].to_string());
            remaining = &remaining[end + "```".len()..];
        } else {
            reasoning_parts.push(remaining.to_string());
            remaining = "";
            break;
        }
    }

    if reasoning_parts.is_empty() {
        return None;
    }

    clean.push_str(remaining);
    Some(Extracted {
        reasoning: reasoning_parts.join("\n"),
        clean_text: clean,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_thinking_tags() {
        let result =
            extract_thinking_tags("Hello <thinking>let me think</thinking> world").unwrap();
        assert_eq!(result.reasoning, "let me think");
        assert_eq!(result.clean_text, "Hello  world");

        // no tags
        assert!(extract_thinking_tags("no thinking here").is_none());

        // unclosed tag
        let result = extract_thinking_tags("Hi <thinking>unclosed").unwrap();
        assert_eq!(result.reasoning, "unclosed");
        assert_eq!(result.clean_text, "Hi ");
    }

    #[test]
    fn test_extract_reasoning_fences() {
        let result =
            extract_reasoning_fences("text ```reasoning\nthink step\nby step\n``` more").unwrap();
        assert_eq!(result.reasoning, "\nthink step\nby step\n");
        assert_eq!(result.clean_text, "text  more");

        // no fences
        assert!(extract_reasoning_fences("plain text").is_none());
    }

    #[test]
    fn test_multiple_thinking_blocks() {
        let result =
            extract_thinking_tags("a <thinking>first</thinking> b <thinking>second</thinking> c")
                .unwrap();
        assert_eq!(result.reasoning, "first\nsecond");
        assert_eq!(result.clean_text, "a  b  c");
    }
}
