//! Prompt types.
use serde::{Deserialize, Serialize};

use crate::message::Message;

/// A prompt that can be either raw text or a list of messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Prompt {
    /// A plain text prompt.
    Text(String),
    /// A list of conversation messages.
    Messages(Vec<Message>),
}

impl Prompt {
    /// Convert the prompt into a `Vec<Message>`.
    /// A plain text prompt becomes a single user message.
    pub fn into_messages(self) -> Vec<Message> {
        match self {
            Prompt::Text(text) => vec![Message::user(text)],
            Prompt::Messages(msgs) => msgs,
        }
    }
}

impl From<&str> for Prompt {
    fn from(s: &str) -> Self {
        Prompt::Text(s.to_owned())
    }
}

impl From<String> for Prompt {
    fn from(s: String) -> Self {
        Prompt::Text(s)
    }
}

impl From<Vec<Message>> for Prompt {
    fn from(msgs: Vec<Message>) -> Self {
        Prompt::Messages(msgs)
    }
}
