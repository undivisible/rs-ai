use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::content::{ContentPart, ImageData};

/// The role of a message participant.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Message {
    /// Create a system message from plain text.
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: vec![ContentPart::Text { text: text.into() }],
            name: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a user message from plain text.
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentPart::Text { text: text.into() }],
            name: None,
            metadata: HashMap::new(),
        }
    }

    /// Create an assistant message from plain text.
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentPart::Text { text: text.into() }],
            name: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a tool-result message.
    pub fn tool_result(call_id: impl Into<String>, content: impl Into<String>) -> Self {
        use crate::tool::ToolCallResult;
        Self {
            role: Role::Tool,
            content: vec![ContentPart::ToolResult {
                result: ToolCallResult {
                    call_id: call_id.into(),
                    content: content.into(),
                    is_error: false,
                },
            }],
            name: None,
            metadata: HashMap::new(),
        }
    }

    /// Append an image to this message's content parts.
    pub fn with_image(mut self, image_data: ImageData) -> Self {
        self.content.push(ContentPart::Image { data: image_data });
        self
    }

    /// Set the `name` field on this message.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Insert an arbitrary metadata key/value.
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}
