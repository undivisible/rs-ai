//! Multimodal content parts for messages.
use serde::{Deserialize, Serialize};

use crate::tool::{ToolCallRequest, ToolCallResult};

/// A single part of a message's content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// Plain text content.
    Text {
        /// The text string.
        text: String,
    },
    /// Image content.
    Image {
        /// The image payload.
        data: ImageData,
    },
    /// File content.
    File {
        /// The file payload.
        data: FileData,
    },
    /// A request from the model to call a tool.
    ToolCall {
        /// The tool call request.
        call: ToolCallRequest,
    },
    /// The result of a tool execution.
    ToolResult {
        /// The tool call result.
        result: ToolCallResult,
    },
}

/// Image payload — either a URL reference or inline base64.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum ImageData {
    /// Reference an image by URL.
    Url {
        /// URL of the image.
        url: String,
        /// Desired detail level.
        detail: Option<ImageDetail>,
    },
    /// Inline base64-encoded image data.
    Base64 {
        /// MIME type of the image.
        media_type: String,
        /// Base64-encoded image bytes.
        data: String,
    },
}

/// Requested level of detail for image understanding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageDetail {
    /// Let the model decide the detail level.
    Auto,
    /// Low detail (faster, cheaper).
    Low,
    /// High detail (more tokens).
    High,
}

/// File payload — either a URL reference or inline base64.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum FileData {
    /// Reference a file by URL.
    Url {
        /// URL of the file.
        url: String,
        /// MIME type of the file.
        media_type: Option<String>,
    },
    /// Inline base64-encoded file data.
    Base64 {
        /// MIME type of the file.
        media_type: String,
        /// Base64-encoded file bytes.
        data: String,
    },
}
