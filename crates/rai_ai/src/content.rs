use serde::{Deserialize, Serialize};

use crate::tool::{ToolCallRequest, ToolCallResult};

/// A single part of a message's content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text { text: String },
    Image { data: ImageData },
    File { data: FileData },
    ToolCall { call: ToolCallRequest },
    ToolResult { result: ToolCallResult },
}

/// Image payload — either a URL reference or inline base64.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum ImageData {
    Url {
        url: String,
        detail: Option<ImageDetail>,
    },
    Base64 {
        media_type: String,
        data: String,
    },
}

/// Requested level of detail for image understanding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageDetail {
    Auto,
    Low,
    High,
}

/// File payload — either a URL reference or inline base64.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum FileData {
    Url {
        url: String,
        media_type: Option<String>,
    },
    Base64 {
        media_type: String,
        data: String,
    },
}
